use rocket::{serde::json::Json, http::Status, State, routes, get, post};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Emitter}; // 需要重新引入 AppHandle, Manager
use log::{info, error};
use rocket_cors::{AllowedOrigins, CorsOptions};

use crate::account::{self, Account, get_account_manager, ApiResponse};

const SERVER_PORT: u16 = 34999;

// 定义请求和响应的结构体
#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    access_token: String,
    device_user_id: Option<String>,
}

// 服务器状态 (与之前相同，但需要确保 AppHandle 可用)
struct ServerState {
    app_handle: AppHandle,
    // running 状态在 Rocket 中通常不需要显式管理，服务启动即运行
}

impl ServerState {
    fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

// Rocket 路由处理函数
// 使用 `#[post(...)]` 宏定义路由和方法
// 返回类型需要调整为 Rocket 的形式，通常是 (Status, Json<T>) 或 Result<Json<T>, Status> 等
#[post("/receive_login", format = "json", data = "<payload>")]
async fn handle_login(
    payload: Json<LoginRequest>,
    app_handle: &State<AppHandle>,          // 只使用 AppHandle
) -> (Status, Json<ApiResponse<Account>>) {
    let request = payload.into_inner(); // 获取 Json 内部的数据

    info!("收到登录请求: access_token={}, device_user_id={:?}",
        if request.access_token.is_empty() { "[空]" } else { "[已提供]" },
        request.device_user_id);

    if request.access_token.is_empty() {
        return (
            Status::BadRequest,
            Json(ApiResponse {
                code: "invalid_params".to_string(),
                data: Account::default(),
                errorMsg: "访问令牌不能为空".to_string(),
                fail: true,
                success: false,
            }),
        );
    }

    // 直接使用注入的 AppHandle
    let handle = app_handle.inner().clone();

    match login_by_token(request.access_token, request.device_user_id, handle).await {
        Ok(account) => {
            // 账号状态已在update_account方法中发送事件，此处无需重复发送
            info!("登录成功，account-status-changed事件已由update_account发送");

            (
                Status::Ok,
                Json(ApiResponse {
                    code: "success".to_string(),
                    data: account,
                    errorMsg: "".to_string(),
                    fail: false,
                    success: true,
                }),
            )
        },
        Err(e) => {
            error!("登录失败: {}", e);
            (
                Status::InternalServerError,
                Json(ApiResponse {
                    code: "login_failed".to_string(),
                    data: Account::default(),
                    errorMsg: e,
                    fail: true,
                    success: false,
                }),
            )
        }
    }
}

// 实现通过Token登录的功能 (与之前相同，保持不变)
async fn login_by_token(access_token: String, device_user_id: Option<String>, app_handle: AppHandle) -> Result<Account, String> {
    let account_manager = get_account_manager();
    
    // 准备请求参数
    let mut request_body = serde_json::json!({
        "accessToken": access_token
    });
    
    if let Some(device_id) = device_user_id {
        request_body["deviceUserId"] = serde_json::Value::String(device_id);
    }
    
    // 创建HTTP客户端
    let client = reqwest::Client::new();
    
    // 发送请求到服务端API
    let response = match client
        .post(&format!("{}/api/account/loginByToken", crate::account::API_BASE_URL))
        .json(&request_body)
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => return Err(format!("网络请求失败: {}", e)),
        };
        
    // 检查HTTP状态码
    if !response.status().is_success() {
        return Err(format!("登录失败，HTTP状态码: {}", response.status()));
    }
    
    // 解析响应文本
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => return Err(format!("读取响应内容失败: {}", e)),
    };
    
    info!("Token登录响应: {}", response_text);
    
    // 解析API响应
    let api_response: ApiResponse<Account> = match serde_json::from_str(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            error!("解析登录响应失败，原始响应: {}", response_text);
            return Err(format!("解析响应失败: {}", e));
        }
    };
    
    if !api_response.success {
        return Err(format!("登录失败: {}", api_response.errorMsg));
    }
    
    let account_data = api_response.data;
    
    // 使用公共方法更新账号信息
    if let Err(e) = account_manager.update_account(account_data.clone(), Some(&app_handle)) {
        return Err(format!("更新账号信息失败: {}", e));
    }
    
    info!("Token登录成功");
    Ok(account_data)
}

// 配置 CORS
fn rocket_cors_options() -> rocket_cors::Cors {
    let allowed_origins = AllowedOrigins::all(); // 允许所有来源，或者根据需要配置
    CorsOptions {
        allowed_origins,
        // allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(), // 默认允许 GET, POST, HEAD
        // allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]), // 根据需要配置
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error creating CORS fairing")
}

// 启动 Rocket 服务器的函数
// 注意：Rocket 的启动通常是阻塞的，或者需要在单独的 Tokio 任务中运行
// 这里我们将其修改为在后台启动
pub fn start_login_server(app_handle: AppHandle) {
    let app_handle_clone = app_handle.clone(); // 克隆 AppHandle 用于 Rocket 状态

    // 在 Tokio 运行时中启动 Rocket 服务器
    tokio::spawn(async move {
        info!("启动 Rocket 登录服务器 http://localhost:{}/receive_login", SERVER_PORT);

        let figment = rocket::Config::figment()
            .merge(("port", SERVER_PORT))
            .merge(("address", "127.0.0.1"));

        let rocket_instance = rocket::custom(figment)
            .mount("/", routes![handle_login]) // 挂载路由
            .manage(app_handle_clone)        // 只管理 AppHandle 状态
            .attach(rocket_cors_options());  // 附加 CORS Fairing

        // 启动 Rocket 服务器
        if let Err(e) = rocket_instance.launch().await {
            error!("Rocket 服务器启动失败: {}", e);
        }
    });
}

// 注意：如果之前有 /status 路由，需要用 Rocket 的方式重新实现，或暂时移除
// #[get("/status")]
// fn handle_status(state: &State<Arc<Mutex<ServerState>>>) -> Json<ApiResponse<String>> {
//     let running = *state.lock().unwrap().running.lock().unwrap(); // 访问状态
//     Json(ApiResponse {
//         code: "success".to_string(),
//         data: format!("服务器正在运行: {}", running),
//         errorMsg: "".to_string(),
//         fail: false,
//         success: true,
//     })
// } 