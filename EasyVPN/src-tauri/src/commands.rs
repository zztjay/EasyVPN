// 最简单的命令模块，仅包含基本的greet命令

// 导入必要的模块
use tauri::{AppHandle, Wry};
use crate::clash::{self, ClashMode};
use crate::common::{ProxyCheckCode, AccountStatus};
use crate::account;
/// 启动Clash并设置系统代理
#[tauri::command]
     pub fn start_clash(app_handle: AppHandle<Wry>) -> Result<(), String> {
         println!("commands::start_clash被调用了");
         clash::start_clash_and_proxy(&app_handle).map_err(|e| e.to_string())
     }

/// 停止Clash并关闭系统代理
#[tauri::command]
pub fn stop_clash(app_handle: AppHandle<Wry>) -> Result<(), String> {
    clash::stop_clash_and_proxy(&app_handle).map_err(|e| e.to_string())
}

/// 设置Clash为连接模式（Rule模式）
#[tauri::command]
pub async fn connect_vpn(app_handle: AppHandle<Wry>, restart: bool) -> Result<(), String> {
    // 处理重启
    if restart {
        println!("重新启动 Clash 和代理...");
        if let Err(e) = clash::start_clash_and_proxy(&app_handle) {
            return Err(format!("重新启动Clash失败: {}", e));
        }
    }
    
    // 设置模式
    println!("设置 Clash 模式为 Rule...");
    match clash::set_mode(ClashMode::Rule).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("设置Clash模式失败: {}", e))
    }
}

/// 设置Clash为断开模式（Direct模式）
#[tauri::command]
pub async fn disconnect_vpn() -> Result<(), String> {
    clash::set_mode(ClashMode::Direct).await.map_err(|e| e.to_string())
}

/// 获取Clash状态
#[tauri::command]
pub async fn get_clash_status() -> Result<serde_json::Value, String> {
    clash::get_status().await.map_err(|e| e.to_string())
} 

#[tauri::command]
pub fn log_to_console(message: String) {
    println!("前端日志: {}", message);
}
#[tauri::command]
pub async fn check_system_proxy() -> Result<ProxyCheckCode, String> {
    // 调用修改后的检查函数
    match clash::check_system_proxy() {
        Ok(code) => {
            println!("系统代理检查结果: {:?} - {}", code, code.get_message());
            Ok(code)
        },
        Err(e) => {
            let error_msg = format!("检查系统代理状态失败: {}", e);
            println!("{}", error_msg);
            Ok(ProxyCheckCode::CheckError)
        }
    }
}

/// 获取账号信息
#[tauri::command]
pub async fn get_account_info() -> Result<account::Account, String> {
    match account::get_current_account().await {
        Ok(account) => Ok(account),
        Err(e) => Err(format!("获取账号信息失败: {}", e))
    }
}

/// 获取账号状态的文本描述
#[tauri::command]
pub fn get_account_status_text(status: AccountStatus) -> String {
    status.get_message().to_string()
}

