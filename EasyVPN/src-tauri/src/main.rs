// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

// 导入我们的命令模块
mod commands;
mod clash;
mod common;
mod account;
use tauri::{AppHandle, Manager};
use tokio::join;

fn main() {
    println!("应用程序启动...");
    
    let tauri_builder = tauri::Builder::default();
    
    tauri_builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            
            let app_handle = app.handle();
            
            // 创建窗口关闭事件监听器
            if let Some(main_window) = app.get_webview_window("main") {
                
                // 设置关闭事件监听器
                let app_handle_clone = app_handle.clone();
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { .. } => {
                            println!("接收到窗口关闭请求");
                            // 在窗口关闭时停止Clash并关闭系统代理
                            if let Err(e) = commands::stop_clash(app_handle_clone.clone()) {
                                eprintln!("关闭Clash时出错: {}", e);
                            }
                        }
                        _ => {}
                    }
                });
            
            }
            
            // 在后台执行初始化流程，完成后再显示窗口
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                println!("正在并行初始化...");
                
                // 并行执行初始化任务
                let clash_task = tauri::async_runtime::spawn({
                    let app_handle = app_handle_clone.clone();
                    async move {
                        if let Err(e) = commands::start_clash(app_handle) {
                            eprintln!("启动Clash失败: {}", e);
                            false
                        } else {
                            println!("Clash启动成功");
                            true
                        }
                    }
                });
                
                let account_task = tauri::async_runtime::spawn({
                    let app_handle = app_handle_clone.clone();
                    async move {
                        if let Err(e) = account::initialize_account(app_handle).await {
                            eprintln!("初始化账号失败: {}", e);
                            false
                        } else {
                            println!("账号初始化成功");
                            true
                        }
                    }
                });
                
                // 等待两个任务完成
                let (_clash_result, _account_result) = join!(clash_task, account_task);
                
                // 初始化完成后显示窗口
                println!("后端初始化完成，准备显示窗口...");
                if let Some(main_window) = app_handle_clone.get_webview_window("main") {
                    // 直接显示窗口，前端通过监听visibilitychange事件知道窗口已显示
                    if let Err(e) = main_window.show() {
                        eprintln!("显示主窗口失败: {}", e);
                    } else {
                        println!("主窗口已显示");
                    }
                }
            });
            
            println!("设置阶段完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 移除start_clash命令，因为现在在初始化时自动启动
            commands::stop_clash,
            commands::connect_vpn,
            commands::disconnect_vpn,
            commands::get_clash_status,
            commands::log_to_console,
            commands::check_system_proxy,
            commands::get_account_info,
            commands::get_traffic,
            commands::speed_test,
            commands::get_speed_test_results,
            account::login,
            account::logout,
            account::get_current_device_info,
            account::update_and_get_account,
        ])
        .run(tauri::generate_context!())
        .expect("应用程序运行失败");
}
