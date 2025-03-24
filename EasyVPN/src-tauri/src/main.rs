// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

// 导入我们的命令模块
mod commands;

fn main() {
    println!("应用程序启动...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet
        ])
        .run(tauri::generate_context!())
        .expect("应用程序运行失败");
}
