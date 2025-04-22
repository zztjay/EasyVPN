use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};
use reqwest::Client;
use tokio::time::{interval, Duration};
use std::io;
use std::error::Error;
use std::fmt;
use tokio::task;
use tauri::{AppHandle, Manager, Emitter}; // Emitter 是关键！
use crate::common::AccountStatus;
use hostname;
use rand;
use machine_uid;

// 服务器域名常量
pub const API_BASE_URL: &str = "http://127.0.0.1:8080";
// 本地账号信息文件路径
const ACCOUNT_FILE_PATH: &str = "account.json";
// 设备ID文件路径
const DEVICE_ID_FILE_PATH: &str = "deviceId.json";

// 定义账号信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(default)]
    pub accountId: String,
    #[serde(default)]
    pub accessToken: String,
    #[serde(default)]
    pub refreshToken: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub guest: bool,                // 是否为游客账号
    #[serde(default)]
    pub serviceExpiryDate: Option<String>, // 服务到期时间
    #[serde(default)]
    pub userId: Option<i64>,        // 用户ID
    #[serde(default)]
    pub username: Option<String>,   // 用户名
    #[serde(default)]
    pub devices: Option<Vec<DeviceInfo>>, // 设备信息
    #[serde(default)]
    pub loginType: Option<String>,  // 登录类型
}

// 默认账号信息
impl Default for Account {
    fn default() -> Self {
        Self {
            accountId: String::new(),
            accessToken: String::new(),
            refreshToken: String::new(),
            status: String::from("UNREGISTERED"),
            guest: false,
            serviceExpiryDate: None,
            userId: None,
            username: None,
            devices: None,
            loginType: None,
        }
    }
}

// 定义API响应结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: String,
    #[serde(default)]
    pub data: T,
    pub errorMsg: String,
    pub fail: bool,
    pub success: bool,
}

// 定义设备信息结构体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInfo {
    #[serde(default)]
    deviceId: i64,
    #[serde(default)]
    deviceName: Option<String>,
    #[serde(default)]
    deviceUserId: i64,
    #[serde(default)]
    lastOnlineTime: String,
    #[serde(default)]
    macAddress: String,
    #[serde(default)]
    active: Option<bool>,
    #[serde(default)]
    trialExpiryDate: Option<String>,
}

// 自定义错误类型
#[derive(Debug)]
pub enum AccountError {
    Io(io::Error),
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    ApiError(String),
    Other(String),
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountError::Io(err) => write!(f, "IO错误: {}", err),
            AccountError::Reqwest(err) => write!(f, "网络请求错误: {}", err),
            AccountError::Json(err) => write!(f, "JSON解析错误: {}", err),
            AccountError::ApiError(msg) => write!(f, "API错误: {}", msg),
            AccountError::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

impl Error for AccountError {}

impl From<io::Error> for AccountError {
    fn from(err: io::Error) -> Self {
        AccountError::Io(err)
    }
}

impl From<reqwest::Error> for AccountError {
    fn from(err: reqwest::Error) -> Self {
        AccountError::Reqwest(err)
    }
}

impl From<serde_json::Error> for AccountError {
    fn from(err: serde_json::Error) -> Self {
        AccountError::Json(err)
    }
}

// 静态内存缓存，用于存储设备ID
lazy_static::lazy_static! {
    static ref DEVICE_ID: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}

// 账号管理器结构体
pub struct AccountManager {
    account: Arc<RwLock<Account>>,
    client: Client,
}

impl AccountManager {
    // 创建新的账号管理器实例
    pub fn new() -> Self {
        Self {
            account: Arc::new(RwLock::new(Account::default())),
            client: Client::new(),
        }
    }

    // 获取当前账号信息的克隆
    pub fn get_account(&self) -> Result<Account, AccountError> {
        match self.account.read() {
            Ok(account) => Ok(account.clone()),
            Err(_) => Err(AccountError::Other("无法读取账号信息".to_string())),
        }
    }

    // 初始化账号信息
    pub async fn initialize(&self, app_handle: Option<&AppHandle>) -> Result<(), AccountError> {
        if Path::new(ACCOUNT_FILE_PATH).exists() {
            // 如果本地文件存在，则读取文件并尝试更新状态
            let account = match self.load_account_from_file() {
                Ok(account) => {
                    println!("成功从文件加载账号信息");
                    account
                },
                Err(e) => {
                    // 如果加载失败，可能是文件损坏，删除文件并注册新账号
                    eprintln!("加载账号文件失败: {}, 将尝试重新注册账号", e);
                    if Path::new(ACCOUNT_FILE_PATH).exists() {
                        if let Err(e) = fs::remove_file(ACCOUNT_FILE_PATH) {
                            eprintln!("删除损坏的账号文件失败: {}", e);
                        }
                    }
                    return self.device_login().await;
                }
            };
            
            // 使用access_token获取最新状态
            match self.update_account_status(app_handle).await {
                Ok(_) => {
                    println!("账号状态更新成功");
                    Ok(())
                },
                Err(e) => {
                    // 如果更新失败，检查是否是Token过期
                    let token_expired = match &e {
                        AccountError::ApiError(msg) => {
                            msg.contains("Token已过期") || msg.contains("无效")
                        },
                        _ => false,
                    };
                    
                    if token_expired {
                        println!("Token已过期，尝试刷新Token");
                        // 尝试刷新token
                        match self.refresh_token(app_handle).await {
                            Ok(_) => {
                                println!("Token刷新成功，重新尝试更新状态");
                                let _ = self.update_account_status(app_handle).await?;
                                Ok(())
                            },
                            Err(refresh_err) => {
                                // 如果刷新也失败，检查是否是refreshToken也过期
                                let refresh_expired = match &refresh_err {
                                    AccountError::ApiError(msg) => {
                                        msg.contains("刷新令牌无效") || msg.contains("已过期")
                                    },
                                    _ => false,
                                };
                                
                                if refresh_expired {
                                    eprintln!("刷新令牌也已过期，将删除本地账号并重新注册");
                                    // 删除本地文件并重新注册
                                    if Path::new(ACCOUNT_FILE_PATH).exists() {
                                        if let Err(e) = fs::remove_file(ACCOUNT_FILE_PATH) {
                                            eprintln!("删除过期账号文件失败: {}", e);
                                        }
                                    }
                                    // 重置内存中的账号为默认值
                                    let default_account = Account::default();
                                    self.update_account(default_account, app_handle)?;
                                    
                                    return self.device_login().await;
                                } else {
                                    // 其他错误，返回原始错误
                                    Err(refresh_err)
                                }
                            }
                        }
                    } else {
                        // 不是Token过期问题，返回原始错误
                        Err(e)
                    }
                }
            }
        } else {
            // 如果本地文件不存在，则注册新账号
            println!("账号文件不存在，将注册新账号");
            self.device_login().await
        }
    }

    // 从文件加载账号信息
    fn load_account_from_file(&self) -> Result<Account, AccountError> {
        let mut file = File::open(ACCOUNT_FILE_PATH)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // 先尝试直接解析为完整的Account结构
        match serde_json::from_str::<Account>(&contents) {
            Ok(account) => {
                // 成功解析完整账号
                Ok(account)
            },
            Err(_) => {
                // 尝试解析为只有token的简化版本
                match serde_json::from_str::<serde_json::Value>(&contents) {
                    Ok(token_data) => {
                        let mut account = Account::default();
                        
                        // 从JSON中提取token值
                        if let Some(access_token) = token_data.get("accessToken").and_then(|v| v.as_str()) {
                            account.accessToken = access_token.to_string();
                        }
                        
                        if let Some(refresh_token) = token_data.get("refreshToken").and_then(|v| v.as_str()) {
                            account.refreshToken = refresh_token.to_string();
                        }
                        
                        Ok(account)
                    },
                    Err(e) => Err(AccountError::Json(e)),
                }
            }
        }
    }

    // 保存账号信息到文件
    pub fn save_account_to_file(&self) -> Result<(), AccountError> {
        let account_read = self.account.read().map_err(|_| {
            AccountError::Other("无法获取账号信息读取锁".to_string())
        })?;
        
        // 创建只包含token的简化版账号对象
        let token_only_account = serde_json::json!({
            "accessToken": account_read.accessToken,
            "refreshToken": account_read.refreshToken
        });
        
        let json = serde_json::to_string_pretty(&token_only_account)?;
        let mut file = File::create(ACCOUNT_FILE_PATH)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    // 获取机器唯一ID
    async fn get_machine_id(&self) -> Result<String, AccountError> {
        // 1. 首先尝试从内存缓存获取
        {
            if let Ok(device_id_guard) = DEVICE_ID.read() {
                if let Some(device_id) = device_id_guard.as_ref() {
                    println!("从内存缓存获取设备ID: {}", device_id);
                    return Ok(device_id.clone());
                }
            }
        }
        
        // 2. 如果内存中没有，尝试从文件获取
        if Path::new(DEVICE_ID_FILE_PATH).exists() {
            match fs::read_to_string(DEVICE_ID_FILE_PATH) {
                Ok(file_content) => {
                    match serde_json::from_str::<serde_json::Value>(&file_content) {
                        Ok(json_data) => {
                            if let Some(device_id) = json_data.get("deviceId").and_then(|v| v.as_str()) {
                                let device_id = device_id.to_string();
                                println!("从文件获取设备ID: {}", device_id);
                                
                                // 将设备ID存入内存缓存
                                if let Ok(mut device_id_guard) = DEVICE_ID.write() {
                                    *device_id_guard = Some(device_id.clone());
                                }
                                
                                return Ok(device_id);
                            }
                        }
                        Err(e) => {
                            eprintln!("解析设备ID文件失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取设备ID文件失败: {}", e);
                }
            }
        }
        
        // 3. 如果文件中没有或读取失败，使用machine-uuid生成
        let device_id = {
            // 处理Result，使用unwrap_or_else提供默认值
            let uuid = machine_uid::get().unwrap_or_else(|e| {
                eprintln!("获取机器ID失败: {:?}", e);
                // 生成随机ID作为后备方案
                format!("random-{}", rand::random::<u64>())
            });
            println!("使用machine-uuid生成设备ID: {}", uuid);
            uuid  // 重要！这里需要返回uuid值
        };
        
        println!("使用machine-uuid生成设备ID: {}", device_id);
        // 保存到文件
        let json_data = serde_json::json!({
            "deviceId": device_id
        });
        
        if let Err(e) = fs::write(DEVICE_ID_FILE_PATH, serde_json::to_string_pretty(&json_data).unwrap_or_default()) {
            eprintln!("保存设备ID到文件失败: {}", e);
        } else {
            println!("设备ID已保存到文件");
        }
        
        // 保存到内存缓存
        if let Ok(mut device_id_guard) = DEVICE_ID.write() {
            *device_id_guard = Some(device_id.clone());
        }
        
        Ok(device_id)
    }

    // 注册新账号
    async fn device_login(&self) -> Result<(), AccountError> {
        let device_id: String = self.get_machine_id().await?;
        
        // 获取系统主机名
        let hostname = match hostname::get() {
            Ok(name) => match name.into_string() {
                Ok(name_str) => format!("{}-EasyVPN", name_str),
                Err(_) => "未知设备-EasyVPN".to_string()
            },
            Err(_) => "未知设备-EasyVPN".to_string()
        };
        
        println!("使用设备名: {}", hostname);
        
        // 发送请求
        let response = match self.client
            .post(&format!("{}/api/account/deviceLogin", API_BASE_URL))
            .json(&serde_json::json!({
                "deviceId": device_id,
                "deviceName": hostname
            }))
            .send()
            .await {
                Ok(resp) => resp,
                Err(e) => return Err(AccountError::Reqwest(e)),
            };
            
        // 检查HTTP状态码
        if !response.status().is_success() {
            return Err(AccountError::ApiError(format!(
                "HTTP请求失败，状态码: {}", response.status()
            )));
        }
        
        // 尝试解析响应为字符串，用于调试和错误处理
        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => return Err(AccountError::Reqwest(e)),
        };
        
        // 尝试将响应解析为API响应结构
        let api_response: ApiResponse<Account> = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("解析API响应失败，原始响应: {}", response_text);
                return Err(AccountError::Json(e));
            }
        };
        
        if !api_response.success {
            return Err(AccountError::ApiError(format!(
                "注册账号失败: {}", api_response.errorMsg
            )));
        }
        
        let account_data = api_response.data;
        
        // 使用update_account方法更新账号信息
        self.update_account(account_data, None)?;
        
        println!("设备登录成功，token已保存到文件");
        Ok(())
    }

    // 更新账号状态
    async fn update_account_status(&self, app_handle: Option<&AppHandle>) -> Result<bool, AccountError> {
        let access_token = {
            let account_read = self.account.read().map_err(|_| {
                AccountError::Other("无法获取账号信息读取锁".to_string())
            })?;
            account_read.accessToken.clone()
        };
        
        if access_token.is_empty() {
            return Err(AccountError::Other("Access token为空".to_string()));
        }
        
        println!("正在更新账号状态...");
        
        // 发送请求
        let response = match self.client
            .post(&format!("{}/api/account/status", API_BASE_URL))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("发送状态请求失败: {}", e);
                    return Err(AccountError::Reqwest(e));
                }
            };
            
        // 检查HTTP状态码
        if !response.status().is_success() {
            let status_code = response.status();
            eprintln!("HTTP请求失败，状态码: {}", status_code);
            
            // 如果是401错误，说明token无效
            if status_code == 401 {
                return Err(AccountError::ApiError("Token已过期或无效".to_string()));
            }
            
            return Err(AccountError::ApiError(format!(
                "HTTP请求失败，状态码: {}", status_code
            )));
        }
        
        // 尝试解析响应为字符串，用于调试和错误处理
        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                eprintln!("读取响应内容失败: {}", e);
                return Err(AccountError::Reqwest(e));
            }
        };
        
        println!("状态更新响应: {}", response_text);
        
        // 尝试将响应解析为API响应结构
        let api_response: ApiResponse<Account> = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("解析API响应失败，原始响应: {}", response_text);
                return Err(AccountError::Json(e));
            }
        };
        
        if !api_response.success {
            // 处理各种错误情况
            eprintln!("API请求失败，错误码: {}, 错误信息: {}", api_response.code, api_response.errorMsg);
            
            if api_response.code == "invalid_token" {
                return Err(AccountError::ApiError("Token已过期或无效".to_string()));
            } else {
                return Err(AccountError::ApiError(format!(
                    "获取账号状态失败: {}", api_response.errorMsg
                )));
            }
        }
        
        let account_data = api_response.data;
        
        
       // 从当前账号获取token信息
        let (current_access_token, current_refresh_token) = {
            let account_read = self.account.read().map_err(|_| {
                AccountError::Other("无法获取账号信息读取锁".to_string())
            })?;
            (account_read.accessToken.clone(), account_read.refreshToken.clone())
        };
    
    // 保留当前token信息（除非新token非空）
    let mut updated_account = account_data;
    if updated_account.accessToken.is_empty() {
        updated_account.accessToken = current_access_token;
    }
    
    if updated_account.refreshToken.is_empty() {
        updated_account.refreshToken = current_refresh_token;
    }
    
    // 使用统一的update_account方法更新账号
    let status_changed = self.update_account(updated_account, app_handle)?;
    
    println!("账号状态更新成功");
    Ok(status_changed)
    }

    // 刷新Token
    async fn refresh_token(&self, app_handle: Option<&AppHandle>) -> Result<bool, AccountError> {
        let refresh_token = {
            let account_read = self.account.read().map_err(|_| {
                AccountError::Other("无法获取账号信息读取锁".to_string())
            })?;
            account_read.refreshToken.clone()
        };
        
        if refresh_token.is_empty() {
            return Err(AccountError::Other("Refresh token为空".to_string()));
        }
        
        println!("正在尝试刷新Token...");
        
        // 发送请求
        let response = match self.client
            .post(&format!("{}/api/account/refresh", API_BASE_URL))
            .json(&serde_json::json!({
                "refreshToken": refresh_token
            }))
            .send()
            .await {
                Ok(resp) => resp,
                Err(e) => return Err(AccountError::Reqwest(e)),
            };
            
        // 检查HTTP状态码
        if !response.status().is_success() {
            return Err(AccountError::ApiError(format!(
                "HTTP请求失败，状态码: {}", response.status()
            )));
        }
        
        // 尝试解析响应为字符串，用于调试和错误处理
        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => return Err(AccountError::Reqwest(e)),
        };
        
        println!("刷新Token响应: {}", response_text);
        
        // 尝试将响应解析为API响应结构
        let api_response: ApiResponse<serde_json::Value> = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("解析API响应失败，原始响应: {}", response_text);
                return Err(AccountError::Json(e));
            }
        };
        
        if !api_response.success {
            // 处理各种错误情况
            if api_response.code == "invalid_token" {
                return Err(AccountError::ApiError("刷新令牌无效或已过期".to_string()));
            } else {
                return Err(AccountError::ApiError(format!(
                    "刷新Token失败: {}", api_response.errorMsg
                )));
            }
        }
        
        // 从响应中仅提取accessToken
        let new_access_token = match api_response.data.get("accessToken") {
            Some(token) => match token.as_str() {
                Some(token_str) => token_str.to_string(),
                None => return Err(AccountError::Other("响应中的accessToken格式无效".to_string())),
            },
            None => return Err(AccountError::Other("响应中不包含accessToken".to_string())),
        };
        
        // 获取当前内存中的完整账号信息
        let current_account = {
            self.account.read().map_err(|_| {
                AccountError::Other("无法获取账号信息读取锁".to_string())
            })?.clone()
        };
        
        // 创建一个新的账号对象，保留所有现有信息，只更新accessToken
        let mut updated_account = current_account;
        updated_account.accessToken = new_access_token;
        
        // 使用update_account方法更新账号信息
        let status_changed = self.update_account(updated_account, app_handle)?;
        
        println!("Token刷新成功并保存到文件");
        Ok(status_changed)
    }

    // 启动定时更新任务
    pub async fn start_status_update_task(self: Arc<Self>, app_handle: tauri::AppHandle) -> Result<(), AccountError> {
        // 创建一个5秒的定时器
        let mut interval = interval(Duration::from_secs(5));
        
        // 使用tokio后台任务运行定时更新
        task::spawn(async move {
            loop {
                interval.tick().await;
                
                // 尝试更新账号状态，传入app_handle用于状态变化时发送事件
                match self.update_account_status(Some(&app_handle)).await {
                    Ok(status_changed) => {
                        if status_changed {
                            println!("账号状态已更新并发送变更事件");
                        } else {
                            println!("账号状态已检查，无变化");
                        }
                    }
                    Err(e) => {
                        if let AccountError::ApiError(msg) = &e {
                            if msg.contains("Token已过期") || msg.contains("无效") {
                                // 如果Token过期，尝试刷新
                                match self.refresh_token(Some(&app_handle)).await {
                                    Ok(token_status_changed) => {
                                        // 刷新成功后再次尝试更新状态
                                        match self.update_account_status(Some(&app_handle)).await {
                                            Ok(status_changed) => {
                                                if token_status_changed || status_changed {
                                                    println!("Token已刷新，账号状态已更新并发送变更事件");
                                                } else {
                                                    println!("Token已刷新，账号状态已检查，无变化");
                                                }
                                            },
                                            Err(e) => {
                                                eprintln!("刷新Token后仍无法更新状态: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("刷新Token失败: {}", e);
                                    }
                                }
                            } else {
                                eprintln!("更新账号状态失败: {}", e);
                            }
                        } else {
                            eprintln!("更新账号状态失败: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    // 更新账号信息
    pub fn update_account(&self, mut new_account: Account, app_handle: Option<&AppHandle>) -> Result<bool, AccountError> {
        let mut status_changed = false;
        let previous_status;
        
        // 获取当前账号信息的状态用于比较
        {
            let account_read = self.account.read().map_err(|_| {
                AccountError::Other("无法获取账号信息读取锁".to_string())
            })?;
            previous_status = account_read.status.clone();
            
            // 如果新账号的token为空，则保留当前token
            if new_account.accessToken.is_empty() {
                new_account.accessToken = account_read.accessToken.clone();
            }
            
            if new_account.refreshToken.is_empty() {
                new_account.refreshToken = account_read.refreshToken.clone();
            }
        }
        
        // 更新账号信息
        {
            let mut account_write = self.account.write().map_err(|_| {
                AccountError::Other("无法获取账号信息写入锁".to_string())
            })?;
            
            // 检查状态是否变化
            status_changed = previous_status != new_account.status;
            
            // 更新账号信息
            *account_write = new_account;
            
            // 保存到文件
            if let Err(e) = self.save_account_to_file() {
                eprintln!("保存账号信息到文件失败: {}", e);
                // 继续执行，不返回错误，因为内存已更新成功
            }
        }
        
        // 如果状态变化且提供了AppHandle，发送状态变更事件
        if status_changed && app_handle.is_some() {
            if let Some(handle) = app_handle {
                if let Some(window) = handle.get_webview_window("main") {
                    // 获取更新后的账号信息
                    if let Ok(updated_account) = self.get_account() {
                        let _ = window.emit("account-status-changed", &updated_account);
                        println!("已发送账号状态变更事件: {}", updated_account.status);
                    }
                }
            }
        }
        
        // 返回状态是否变化
        Ok(status_changed)
    }
}

// 创建全局账号管理器实例
lazy_static::lazy_static! {
    static ref ACCOUNT_MANAGER: Arc<AccountManager> = Arc::new(AccountManager::new());
}

// 获取全局账号管理器实例的引用
pub fn get_account_manager() -> Arc<AccountManager> {
    ACCOUNT_MANAGER.clone()
}

// 初始化账号管理器并启动定时更新任务
pub async fn initialize_account(app_handle: tauri::AppHandle) -> Result<(), AccountError> {
    let account_manager = get_account_manager();
    
    // 尝试初始化账号
    match account_manager.initialize(Some(&app_handle)).await {
        Ok(_) => {
            println!("账号初始化成功");
            // 启动定时更新任务
            match account_manager.clone().start_status_update_task(app_handle).await {
                Ok(_) => {
                    println!("账号状态更新任务已启动");
                    Ok(())
                },
                Err(e) => {
                    eprintln!("启动账号状态更新任务失败: {}", e);
                    // 但不阻止应用启动
                    Ok(())
                }
            }
        },
        Err(e) => {
            // 如果是网络错误，则可能是服务器不可用，但不阻止应用启动
            match e {
                AccountError::Reqwest(reqwest_err) => {
                    eprintln!("初始化账号时网络错误: {}, 应用将以离线模式启动", reqwest_err);
                    // 仍然尝试启动定时更新任务，它会定期尝试连接
                    let _ = account_manager.clone().start_status_update_task(app_handle).await;
                    Ok(())
                },
                _ => {
                    eprintln!("初始化账号失败: {}", e);
                    // 非网络错误也不阻止应用启动
                    Ok(())
                }
            }
        }
    }
}

// 提供给其他模块获取当前账号信息的函数
pub async fn get_current_account() -> Result<Account, AccountError> {
    get_account_manager().get_account()
}

// 定义用户登录请求结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub deviceUserId: Option<String>,  // 将deviceId改为deviceUserId
}

// 用户登录接口
#[tauri::command]
pub async fn login(auth: AuthRequest, app_handle: AppHandle) -> Result<String, String> {
    let account_manager = get_account_manager();
    let device_id = match auth.deviceUserId {
        Some(id) => id,
        None => match account_manager.get_machine_id().await {
            Ok(id) => id,
            Err(e) => return Err(format!("获取设备ID失败: {}", e)),
        },
    };
    
    // 获取系统主机名
    let hostname = match hostname::get() {
        Ok(name) => match name.into_string() {
            Ok(name_str) => format!("{}-EasyVPN", name_str),
            Err(_) => "未知设备-EasyVPN".to_string()
        },
        Err(_) => "未知设备-EasyVPN".to_string()
    };
    
    println!("登录使用设备名: {}", hostname);
    
    let client = Client::new();
    
    // 发送请求 - 将deviceId改为deviceUserId
    let response = match client
        .post(&format!("{}/api/account/login", API_BASE_URL))
        .json(&serde_json::json!({
            "username": auth.username,
            "password": auth.password,
            "deviceId": device_id, 
            "deviceName": hostname
        }))
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("网络请求失败: {}", e)),
        };
        
    if !response.status().is_success() {
        return Err(format!("登录失败，HTTP状态码: {}", response.status()));
    }
    
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => return Err(format!("读取响应失败: {}", e)),
    };
    
    println!("登录响应: {}", response_text);
    
    let api_response: ApiResponse<Account> = match serde_json::from_str(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("解析登录响应失败，原始响应: {}", response_text);
            return Err(format!("解析响应失败: {}，原始响应: {}", e, response_text));
        }
    };
    
    if !api_response.success {
        return Err(format!("登录失败: {}", api_response.errorMsg));
    }
    
    // 获取账号数据
    let account_data = api_response.data;
    
    // 更新账号信息
    match account_manager.update_account(account_data.clone(), Some(&app_handle)) {
        Ok(_) => {
            println!("登录成功，账号信息已更新");
            
            let account_json = match serde_json::to_string(&account_data) {
                Ok(json) => json,
                Err(e) => return Err(format!("序列化账号信息失败: {}", e)),
            };
            
            Ok(account_json)
        },
        Err(e) => Err(format!("更新账号信息失败: {}", e)),
    }
}

// 退出登录接口
#[tauri::command]
pub async fn logout(app_handle: AppHandle) -> Result<String, String> {
    let account_manager = get_account_manager();
    
    // 获取当前账号的访问令牌和设备ID
    let (access_token, device_user_id) = {
        match account_manager.account.read() {
            Ok(account) => {
                let device_id = if let Some(devices) = &account.devices {
                    if !devices.is_empty() {
                        // 将整数deviceUserId转换为字符串
                        devices[0].deviceUserId.to_string()
                    } else {
                        return Err("找不到设备ID信息".to_string());
                    }
                } else {
                    return Err("账号未绑定设备".to_string());
                };
                
                if account.accessToken.is_empty() {
                    return Err("访问令牌为空，请先登录".to_string());
                }
                
                (account.accessToken.clone(), device_id)
            },
            Err(_) => return Err("无法读取账号信息".to_string()),
        }
    };
    
    // 创建HTTP客户端
    let client = Client::new();
    
    // 发送请求到正确的API端点
    let response = match client
        .post(&format!("{}/api/account/logout", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&serde_json::json!({
            "deviceUserId": device_user_id
        }))
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("网络请求失败: {}", e)),
        };
    
    // 检查HTTP状态码
    if !response.status().is_success() {
        return Err(format!("退出登录失败，HTTP状态码: {}", response.status()));
    }
    
    // 解析响应内容
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => return Err(format!("读取响应内容失败: {}", e)),
    };
    
    println!("退出登录响应: {}", response_text);
    
    // 尝试解析响应
    let api_response: ApiResponse<bool> = match serde_json::from_str(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("解析退出登录响应失败，原始响应: {}", response_text);
            return Err(format!("解析响应失败: {}，原始响应: {}", e, response_text));
        }
    };
    
    if !api_response.success {
        return Err(format!("退出登录失败: {}", api_response.errorMsg));
    } 
    
    // 重新设备登录
    match account_manager.device_login().await {
        Ok(_) => {
            println!("设备重新登录成功");
        },
        Err(e) => {
            eprintln!("设备重新登录失败: {}", e);
            return Err(format!("设备重新登录失败: {}", e));
        }
    }
    
    // 返回成功消息
    Ok("已成功退出登录并重新设备登录".to_string())
}

// 解绑设备接口
#[tauri::command]
pub async fn unbind_device(device_user_id: String, current_device_user_id: Option<String>, app_handle: AppHandle) -> Result<String, String> {
    let account_manager = get_account_manager();
    let access_token = match account_manager.account.read() {
        Ok(account) => account.accessToken.clone(),
        Err(_) => return Err("无法读取账号信息".to_string()),
    };
    
    if access_token.is_empty() {
        return Err("未登录状态，请先登录".to_string());
    }
    
    let client = Client::new();
    
    // 构建请求数据
    let mut request_data = serde_json::json!({
        "deviceUserId": device_user_id
    });
    
    // 如果提供了当前设备ID，添加到请求数据中
    if let Some(current_id) = current_device_user_id {
        request_data["currentDeviceUserId"] = serde_json::Value::String(current_id);
    }
    
    // 发送请求
    let response = match client
        .post(&format!("{}/api/account/unbind-device", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&request_data)
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("网络请求失败: {}", e)),
        };
        
    // 检查HTTP状态码
    if !response.status().is_success() {
        return Err(format!("解绑设备失败，HTTP状态码: {}", response.status()));
    }
    
    // 先获取响应文本
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => return Err(format!("读取响应内容失败: {}", e)),
    };
    
    // 尝试解析响应
    let api_response: ApiResponse<bool> = match serde_json::from_str(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("解析解绑设备响应失败，原始响应: {}", response_text);
            return Err(format!("解析响应失败: {}，原始响应: {}", e, response_text));
        }
    };
    
    if !api_response.success {
        return Err(format!("解绑设备失败: {}", api_response.errorMsg));
    }
    
    // 解绑成功后更新账号状态
    if let Err(e) = account_manager.update_account_status(Some(&app_handle)).await {
        return Err(format!("更新账号状态失败: {}", e));
    }
    
    // 返回成功消息
    Ok("设备解绑成功".to_string())
} 