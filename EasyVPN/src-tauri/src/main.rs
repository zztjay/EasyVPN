// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

// 导入我们的命令模块
mod commands;
mod clash;
mod common;
mod account;
mod web_login;

use tauri::Manager;

fn main() {
    println!("应用程序启动...");
    
    let tauri_builder = tauri::Builder::default();
    println!("Tauri Builder已创建");
    
    tauri_builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            println!("设置阶段开始...");
            
            // 在应用启动时启动Clash
            let app_handle = app.handle();
            
            // 检查所有窗口
            println!("检查应用中的所有窗口...");
            for window in app.webview_windows().values() {
                println!("找到窗口: {}", window.label());
            }
            
            println!("正在获取主窗口...");
            // 创建窗口关闭事件监听器
            if let Some(main_window) = app.get_webview_window("main") {
                println!("成功找到主窗口，设置关闭事件监听器");
                
                if let Ok(url) = main_window.url() {
                    println!("主窗口URL: {:?}", url);
                }
                
                if let Ok(title) = main_window.title() {
                    println!("主窗口标题: {}", title);
                }
                
                if let Ok(visible) = main_window.is_visible() {
                    println!("主窗口是否可见: {}", visible);
                }
                
                let app_handle_clone = app_handle.clone();
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { .. } => {
                            println!("接收到窗口关闭请求");
                            // 在窗口关闭时停止Clash并关闭系统代理
                            if let Err(e) = clash::stop_clash_and_proxy(&app_handle_clone) {
                                eprintln!("关闭Clash时出错: {}", e);
                            }
                        },
                        tauri::WindowEvent::Focused(focused) => {
                            println!("窗口焦点状态变更: {}", focused);
                        },
                        tauri::WindowEvent::Resized(..) => {
                            println!("窗口大小已调整");
                        },
                        _ => {}
                    }
                });
                
                // 尝试显示主窗口
                if let Err(e) = main_window.show() {
                    eprintln!("显示主窗口失败: {}", e);
                } else {
                    println!("已尝试显示主窗口");
                }
                
            } else {
                eprintln!("警告：无法获取主窗口实例！");
            }
            
            println!("正在初始化账号...");
            // 初始化账号
            tauri::async_runtime::block_on(async {
                if let Err(e) = account::initialize_account(app_handle.clone()).await {
                    eprintln!("初始化账号失败: {}", e);
                } else {
                    println!("账号初始化成功");
                }
            });
            
            println!("正在启动Web登录HTTP服务器...");
            // 启动Web登录HTTP服务器
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                web_login::start_login_server(app_handle_clone);
                println!("Web登录服务已启动");
            });
            
            println!("设置阶段完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_clash,
            commands::stop_clash,
            commands::connect_vpn,
            commands::disconnect_vpn,
            commands::get_clash_status,
            commands::log_to_console,
            commands::check_system_proxy,
            commands::get_account_info,
            commands::get_account_status_text,
            account::login,
            account::logout,
            account::unbind_device,
        ])
        .run(tauri::generate_context!())
        .expect("应用程序运行失败");
}
