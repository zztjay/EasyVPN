// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

// 导入我们的命令模块
mod commands;
mod clash;

use tauri::Manager;

fn main() {
    println!("应用程序启动...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 在应用启动时启动Clash
            let app_handle = app.handle();
            
            // 创建窗口关闭事件监听器
            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle_clone = app_handle.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        // 在窗口关闭时停止Clash并关闭系统代理
                        if let Err(e) = clash::stop_clash_and_proxy(&app_handle_clone) {
                            eprintln!("关闭Clash时出错: {}", e);
                        }
                    }
                });
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::start_clash,
            commands::stop_clash,
            commands::connect_vpn,
            commands::disconnect_vpn,
            commands::get_clash_status
        ])
        .run(tauri::generate_context!())
        .expect("应用程序运行失败");
}
