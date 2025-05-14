use serde::{Deserialize, Serialize, Deserializer};
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
use std::path::PathBuf;

// 服务器域名常量
pub const API_BASE_URL: &str = "http://localhost:8080";
// 文件名常量 - 只保留文件名部分
const ACCOUNT_FILENAME: &str = "account.json";
const DEVICE_ID_FILENAME: &str = "deviceId.json";

// 获取应用程序数据目录下的完整文件路径
fn get_app_data_file(app_handle: &tauri::AppHandle, filename: &str) -> Result<PathBuf, AccountError> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| AccountError::Other(format!("无法获取应用数据目录: {}", e)))?;
    
    // 确保目录存在
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AccountError::Io(e))?;
    }
    
    Ok(app_data_dir.join(filename))
}

// 获取账号文件路径
fn get_account_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, AccountError> {
    get_app_data_file(app_handle, ACCOUNT_FILENAME)
}

// 获取设备ID文件路径
fn get_device_id_file_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, AccountError> {
    get_app_data_file(app_handle, DEVICE_ID_FILENAME)
}

// 定义一个辅助函数，将null值转换为空字符串
fn empty_string_as_none<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

// 定义账号信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub accessToken: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub refreshToken: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub serviceExpiryDate: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub devices: Option<Vec<DeviceInfo>>,
    #[serde(default)]
    pub loginType: Option<String>,
    #[serde(default)]
    pub maxDevicesAllowed: i32,
    #[serde(default)]
    pub currentPackageType: Option<String>,
    #[serde(default)]
    pub remainingDays: Option<i32>,
}

// 默认账号信息
impl Default for Account {
    fn default() -> Self {
        Self {
            accessToken: String::new(),
            refreshToken: String::new(),
            status: String::from("NO_INIT"),
            serviceExpiryDate: None,
            username: None,
            devices: None,
            maxDevicesAllowed: 3,
            loginType: Some(String::from("Device")),
            currentPackageType: Some(String::from("免费版")),
            remainingDays: Some(0),
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
    pub deviceId: String,
    #[serde(default)]
    pub deviceName: Option<String>,
    #[serde(default)]
    pub deviceUserId: i64,
    #[serde(default)]
    pub lastOnlineTime: String,
    #[serde(default)]
    pub trialExpiryDate: Option<String>,
}

// 为DeviceInfo添加辅助方法
impl DeviceInfo {
    // 判断该设备是否为当前设备
    pub fn is_current_device(&self, current_device_id: &str) -> bool {
        self.deviceUserId.to_string() == current_device_id
    }
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
            Err(_) => Ok(Account::default()) // 如果读取失败，返回默认值
        }
    }

    // 初始化方法
    pub async fn initialize(&self, app_handle: Option<&AppHandle>) -> Result<(), AccountError> {
        if let Some(handle) = app_handle {
            // 直接使用 deviceLogin
            self.device_login(handle).await?;
        }
        Ok(())
    }

    // 更新账号状态
    pub async fn update_account_status(&self, app_handle: &AppHandle) -> Result<(), AccountError> {
        // 直接使用 deviceLogin 获取最新状态
        self.device_login(app_handle).await
    }

    // deviceLogin 方法
    async fn device_login(&self, app_handle: &AppHandle) -> Result<(), AccountError> {
        let machine_id = self.get_machine_id(app_handle).await?;
        let hostname = Self::get_hostname();
        
        let request_body = serde_json::json!({
            "deviceId": machine_id,
            "deviceName": hostname
        });

        let response = self.client
            .post(&format!("{}/api/account/deviceLogin", API_BASE_URL))
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AccountError::ApiError(format!("设备登录失败: {}", response.status())));
        }

        let api_response: ApiResponse<Account> = response.json().await?;
        
        if !api_response.success {
            return Err(AccountError::ApiError(api_response.errorMsg));
        }

        // 更新账号信息
        self.update_account(api_response.data, Some(app_handle))?;
        
        Ok(())
    }

    // 更新账号信息
    pub fn update_account(&self, mut new_account: Account, app_handle: Option<&AppHandle>) -> Result<(), AccountError> {
        // 更新账号信息
        if let Ok(mut account) = self.account.write() {
            *account = new_account.clone();
        }
        Ok(())
    }

    // 获取当前设备信息
    pub async fn get_current_device(&self, app_handle: &AppHandle) -> Result<Option<DeviceInfo>, AccountError> {
        let machine_id = self.get_machine_id(app_handle).await?;
        let account = self.get_account()?;
        
        if let Some(devices) = account.devices {
            for device in devices {
                if device.deviceId == machine_id {
                    return Ok(Some(device));
                }
            }
        }
        
        Ok(None)
    }

    // 获取机器唯一ID
    pub async fn get_machine_id(&self, app_handle: &AppHandle) -> Result<String, AccountError> {
        // 首先尝试从内存缓存获取
        {
            if let Ok(device_id_guard) = DEVICE_ID.read() {
                if let Some(device_id) = device_id_guard.as_ref() {
                    return Ok(device_id.clone());
                }
            }
        }
        
        // 如果内存中没有，尝试从文件获取
        let device_id_file_path = get_device_id_file_path(app_handle)?;
        if device_id_file_path.exists() {
            match fs::read_to_string(&device_id_file_path) {
                Ok(file_content) => {
                    match serde_json::from_str::<serde_json::Value>(&file_content) {
                        Ok(json_data) => {
                            if let Some(device_id) = json_data.get("deviceId").and_then(|v| v.as_str()) {
                                let device_id = device_id.to_string();
                                
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
        
        // 如果文件中没有或读取失败，生成新的设备ID
        let device_id = {
            let uuid = machine_uid::get().unwrap_or_else(|e| {
                eprintln!("获取机器ID失败: {:?}", e);
                format!("random-{}", rand::random::<u64>())
            });
            uuid
        };
        
        println!("使用machine-uuid生成设备ID: {}", device_id);
        
        // 保存到文件
        let json_data = serde_json::json!({
            "deviceId": device_id
        });
        
        if let Err(e) = fs::write(&device_id_file_path, serde_json::to_string_pretty(&json_data).unwrap_or_default()) {
            eprintln!("保存设备ID到文件失败: {}", e);
        }
        
        // 保存到内存缓存
        if let Ok(mut device_id_guard) = DEVICE_ID.write() {
            *device_id_guard = Some(device_id.clone());
        }
        
        Ok(device_id)
    }

    // 辅助函数：获取主机名（不带EasyVPN后缀）
    fn get_hostname() -> String {
        match hostname::get() {
            Ok(name) => match name.into_string() {
                Ok(name_str) => name_str,
                Err(_) => "未知设备".to_string()
            },
            Err(_) => "未知设备".to_string()
        }
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
            Ok(())
        },
        Err(e) => {
            eprintln!("初始化账号失败: {}", e);
            Ok(()) // 即使失败也返回Ok让应用继续启动
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
        None => match account_manager.get_machine_id(&app_handle).await {
            Ok(id) => id,
            Err(e) => return Err(format!("获取设备ID失败: {}", e)),
        },
    };
    
    // 获取系统主机名（不带后缀）
    let device_name = AccountManager::get_hostname();
    
    let client = Client::new();
    
    // 发送请求 - 将deviceId改为deviceUserId
    let response = match client
        .post(&format!("{}/api/account/login", API_BASE_URL))
        .json(&serde_json::json!({
            "username": auth.username,
            "password": auth.password,
            "deviceId": device_id, 
            "deviceName": device_name
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
    
    
    let api_response: ApiResponse<serde_json::Value> = match serde_json::from_str(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("解析登录响应失败，原始响应: {}", response_text);
            return Err(format!("解析响应失败: {}，原始响应: {}", e, response_text));
        }
    };
    
    if !api_response.success {
        return Err(format!("登录失败: {}", api_response.errorMsg));
    }
    
    // 获取账号数据 - 从Value转换为Account
    let account_data = match serde_json::from_value::<Account>(api_response.data.clone()) {
        Ok(account) => account,
        Err(e) => {
            eprintln!("解析账号数据失败: {}, 原始数据: {:?}", e, api_response.data);
            return Err(format!("解析账号数据失败: {}", e));
        }
    };
    
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
    
    // 获取当前账号的访问令牌和刷新令牌
    let (access_token, refresh_token) = {
        let account_read = match account_manager.account.read() {
            Ok(guard) => guard,
            Err(_) => return Err("无法读取账号信息".to_string()),
        };
        
        (account_read.accessToken.clone(), account_read.refreshToken.clone())
    };
    
    // 获取当前设备信息
    let device_user_id = match account_manager.get_current_device(&app_handle).await {
        Ok(Some(device)) => device.deviceUserId.to_string(),
        Ok(None) => {
            return Err("未找到当前设备信息".to_string());
        },
        Err(e) => return Err(format!("获取当前设备信息失败: {}", e)),
    };
    
    // 创建HTTP客户端
    let client = Client::new();
    
    // 使用URL参数方式传递deviceUserId和refreshToken
    let url = format!("{}/api/account/logout?deviceUserId={}&refreshToken={}", API_BASE_URL, 
    device_user_id, refresh_token);
    
    // 发送请求到API端点
    let response = match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
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
    
    // 尝试解析响应 - 使用serde_json::Value类型支持任何返回数据结构
    let api_response: ApiResponse<serde_json::Value> = match serde_json::from_str(&response_text) {
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
    match account_manager.device_login(&app_handle).await {
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
    
    // 构建URL参数
    let mut url = format!("{}/api/account/unbind-device?deviceUserId={}", API_BASE_URL, device_user_id);
    
    // 如果current_device_user_id有值，添加到URL参数
    if let Some(current_id) = current_device_user_id {
        url = format!("{}&currentDeviceUserId={}", url, current_id);
    }
    
    // 发送请求
    let response = match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
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
    let api_response: ApiResponse<serde_json::Value> = match serde_json::from_str(&response_text) {
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
    if let Err(e) = account_manager.update_account_status(&app_handle).await {
        return Err(format!("更新账号状态失败: {}", e));
    }
    
    // 返回成功消息
    Ok("设备解绑成功".to_string())
}

// 新增Tauri命令，用于获取当前设备信息
#[tauri::command]
pub async fn get_current_device_info(app_handle: AppHandle) -> Result<Option<DeviceInfo>, String> {
    let account_manager = get_account_manager();
    
    match account_manager.get_current_device(&app_handle).await {
        Ok(device) => Ok(device),
        Err(e) => Err(format!("获取当前设备信息失败: {}", e)),
    }
}

// 更新并获取账号信息
#[tauri::command]
pub async fn update_and_get_account(app_handle: AppHandle) -> Result<Account, String> {
    let account_manager = get_account_manager();
    
    // 更新账号状态
    if let Err(e) = account_manager.update_account_status(&app_handle).await {
        return Err(format!("更新账号状态失败: {}", e));
    }
    
    // 获取更新后的账号信息
    match account_manager.get_account() {
        Ok(account) => Ok(account),
        Err(e) => Err(format!("获取账号信息失败: {}", e)),
    }
}
