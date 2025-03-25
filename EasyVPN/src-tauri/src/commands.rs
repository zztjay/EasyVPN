// 最简单的命令模块，仅包含基本的greet命令

// 导入必要的模块
use tauri::{AppHandle, Wry};
use crate::clash::{self, ClashMode};

/// 基本问候命令，用于测试Tauri应用程序
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("你好, {}！来自Rust的问候!", name)
}

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
pub async fn connect_vpn() -> Result<(), String> {
    clash::set_mode(ClashMode::Rule).await.map_err(|e| e.to_string())
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