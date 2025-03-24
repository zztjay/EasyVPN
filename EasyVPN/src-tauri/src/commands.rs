// 最简单的命令模块，仅包含基本的greet命令

/// 基本问候命令，用于测试Tauri应用程序
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("你好, {}！来自Rust的问候!", name)
} 