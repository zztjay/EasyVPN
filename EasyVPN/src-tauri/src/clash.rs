use std::process::{Child, Command};
use std::sync::Mutex;
use std::io::Result;
use tauri::{Wry, AppHandle, Manager};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use once_cell::sync::Lazy;

// 全局单例，存储clash进程
static CLASH_PROCESS: Lazy<Mutex<Option<Child>>> = Lazy::new(|| Mutex::new(None));

// Clash的HTTP API端口
const CLASH_API_PORT: u16 = 9090;

// 用于系统代理的端口设置
const CLASH_PROXY_PORT: u16 = 7890;
const CLASH_SOCKS_PORT: u16 = 7891;

// Clash模式枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClashMode {
    Rule,
    Global,
    Direct,
}

// 启动Clash并设置系统代理
pub fn start_clash_and_proxy(app_handle: &AppHandle<Wry>) -> Result<()> {
    match start_clash(app_handle) {
        Ok(_) => println!("start_clash成功执行"),
        Err(e) => {
            println!("start_clash执行失败: {:?}", e);
            return Err(e);
        }
    }
    
    match set_system_proxy(true) {
        Ok(_) => println!("系统代理设置成功"),
        Err(e) => {
            println!("系统代理设置失败: {:?}", e);
            return Err(e);
        }
    }
    Ok(())
}

// 停止Clash并关闭系统代理
pub fn stop_clash_and_proxy(_app_handle: &AppHandle<Wry>) -> Result<()> {
    set_system_proxy(false)?;
    stop_clash()?;
    Ok(())
}

// 启动Clash
fn start_clash(app_handle: &AppHandle<Wry>) -> Result<()> {
    println!("进入start_clash函数");
    let mut clash_lock = CLASH_PROCESS.lock().unwrap();
    
    // 如果已经启动，则不重复启动
    // 添加额外检查，确保进程真的在运行
    if clash_lock.is_some() {
        // 验证进程是否仍在运行
        if let Some(ref mut child) = *clash_lock {
            match child.try_wait() {
                Ok(None) => {
                    println!("Clash进程已存在且正在运行，不重复启动");
                    return Ok(());
                },
                _ => {
                    // 进程已退出或状态检查失败，清除引用并继续启动新进程
                    println!("检测到已退出的Clash进程引用，清除并重新启动");
                    *clash_lock = None;
                }
            }
        }
    }
    
    // 获取资源路径
    println!("开始获取资源路径");
    let resource_path = match app_handle.path().resource_dir() {
        Ok(path) => {
            println!("资源路径: {:?}", path);
            path
        },
        Err(e) => {
            let err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("无法获取资源路径: {:?}", e)
            );
            println!("资源路径获取失败: {:?}", err);
            return Err(err);
        }
    };
    
    // 修改这里，根据调试输出确定正确的路径
    let bin_dir = if resource_path.to_string_lossy().contains("resources") {
        resource_path.join("bin")
    } else {
        resource_path.join("resources").join("bin")
    };

    let clash_bin_path = bin_dir.join("clash-darwin-arm64");

    let config_dir = if resource_path.to_string_lossy().contains("resources") {
        resource_path.join("config")
    } else {
        resource_path.join("resources").join("config")
    };
    let config_path = config_dir.join("config.yaml");
    
    println!("二进制路径: {:?}", clash_bin_path);
    println!("配置文件路径: {:?}", config_path);
    
    // 确保文件存在
    if !clash_bin_path.exists() {
        let err = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Clash二进制文件不存在: {:?}", clash_bin_path),
        );
        println!("错误: {:?}", err);
        return Err(err);
    }
    
    if !config_path.exists() {
        let err = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Clash配置文件不存在: {:?}", config_path),
        );
        println!("错误: {:?}", err);
        return Err(err);
    }
    
    // 启动Clash进程
    println!("启动Clash进程...");
    
    let child = match Command::new(&clash_bin_path)
        .arg("-f")
        .arg(&config_path)
        .arg("-d")
        .arg(resource_path.join("logs"))
        .spawn() {
            Ok(c) => {
                println!("Clash进程启动成功, PID: {:?}", c.id());
                c
            },
            Err(e) => {
                println!("Clash进程启动失败: {:?}", e);
                return Err(e);
            }
        };
    
    // 存储进程
    *clash_lock = Some(child);
    
    println!("Clash已启动");
    Ok(())
}

// 停止Clash
fn stop_clash() -> Result<()> {
    let mut clash_lock = CLASH_PROCESS.lock().unwrap();
    
    if let Some(ref mut child) = *clash_lock {
        println!("停止Clash...");
        // 发送终止信号
        #[cfg(not(target_os = "windows"))]
        {
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM); }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows上使用taskkill命令终止进程
            let _ = Command::new("taskkill")
                .args(&["/F", "/T", "/PID", &child.id().to_string()])
                .spawn();
        }
        
        // 等待进程退出
        let _ = child.wait();
        println!("Clash已停止");
    }
    
    // 清除进程引用
    *clash_lock = None;
    
    Ok(())
}

// 设置系统代理
fn set_system_proxy(enable: bool) -> Result<()> {
    println!("{}系统代理...", if enable { "启用" } else { "禁用" });
    
    // 根据操作系统执行不同的命令
    #[cfg(target_os = "macos")]
    {
        if enable {
            // 设置HTTP代理
            let _ = Command::new("networksetup")
                .args(&["-setwebproxy", "Wi-Fi", "127.0.0.1", &CLASH_PROXY_PORT.to_string()])
                .output()?;
                
            // 设置HTTPS代理
            let _ = Command::new("networksetup")
                .args(&["-setsecurewebproxy", "Wi-Fi", "127.0.0.1", &CLASH_PROXY_PORT.to_string()])
                .output()?;
                
            // 设置SOCKS代理
            let _ = Command::new("networksetup")
                .args(&["-setsocksfirewallproxy", "Wi-Fi", "127.0.0.1", &CLASH_SOCKS_PORT.to_string()])
                .output()?;
                
            // 启用代理
            let _ = Command::new("networksetup")
                .args(&["-setwebproxystate", "Wi-Fi", "on"])
                .output()?;
            let _ = Command::new("networksetup")
                .args(&["-setsecurewebproxystate", "Wi-Fi", "on"])
                .output()?;
            let _ = Command::new("networksetup")
                .args(&["-setsocksfirewallproxystate", "Wi-Fi", "on"])
                .output()?;
        } else {
            // 关闭代理
            let _ = Command::new("networksetup")
                .args(&["-setwebproxystate", "Wi-Fi", "off"])
                .output()?;
            let _ = Command::new("networksetup")
                .args(&["-setsecurewebproxystate", "Wi-Fi", "off"])
                .output()?;
            let _ = Command::new("networksetup")
                .args(&["-setsocksfirewallproxystate", "Wi-Fi", "off"])
                .output()?;
        }
    }
    
    // 为Windows添加代理设置逻辑
    #[cfg(target_os = "windows")]
    {
        if enable {
            // 设置Windows系统代理
            let proxy_server = format!("127.0.0.1:{}", CLASH_PROXY_PORT);
            let _ = Command::new("reg")
                .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f"])
                .output()?;
            let _ = Command::new("reg")
                .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", "/v", "ProxyServer", "/t", "REG_SZ", "/d", &proxy_server, "/f"])
                .output()?;
        } else {
            // 关闭Windows系统代理
            let _ = Command::new("reg")
                .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f"])
                .output()?;
        }
    }
    
    // 为Linux添加代理设置逻辑
    #[cfg(target_os = "linux")]
    {
        // 根据不同的桌面环境，可以添加更多的设置命令
        if enable {
            // GNOME桌面环境
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
                .output();
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy.http", "host", "127.0.0.1"])
                .output();
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy.http", "port", &CLASH_PROXY_PORT.to_string()])
                .output();
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy.https", "host", "127.0.0.1"])
                .output();
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy.https", "port", &CLASH_PROXY_PORT.to_string()])
                .output();
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"])
                .output();
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy.socks", "port", &CLASH_SOCKS_PORT.to_string()])
                .output();
        } else {
            // 关闭GNOME代理
            let _ = Command::new("gsettings")
                .args(&["set", "org.gnome.system.proxy", "mode", "none"])
                .output();
        }
    }
    
    println!("系统代理已{}设置", if enable { "启用" } else { "禁用" });
    Ok(())
}

// 通过Clash API切换代理模式
pub async fn set_mode(mode: ClashMode) -> Result<()> {
    
    let client = Client::new();
    let mode_str = match mode {
        ClashMode::Rule => "rule",
        ClashMode::Global => "global",
        ClashMode::Direct => "direct",
    };
    
    let response = client.patch(format!("http://127.0.0.1:{}/configs", CLASH_API_PORT))
        .json(&serde_json::json!({
            "mode": mode_str
        }))
        .send()
        .await;
    
    match response {
        Ok(_) => {
            println!("Clash模式已设置为: {}", mode_str);
            Ok(())
        },
        Err(e) => {
            eprintln!("设置Clash模式失败: {}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("设置Clash模式失败: {}", e),
            ))
        }
    }
}

// 获取Clash当前状态
pub async fn get_status() -> Result<serde_json::Value> {
    let client = Client::new();
    let response = client.get(format!("http://127.0.0.1:{}/configs", CLASH_API_PORT))
        .send()
        .await;
    
    match response {
        Ok(res) => {
            match res.json::<serde_json::Value>().await {
                Ok(json) => Ok(json),
                Err(e) => {
                    eprintln!("解析Clash状态失败: {}", e);
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("解析Clash状态失败: {}", e),
                    ))
                }
            }
        },
        Err(e) => {
            eprintln!("获取Clash状态失败: {}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("获取Clash状态失败: {}", e),
            ))
        }
    }
} 

// 修改代理检查函数，返回错误码而不是布尔值
pub fn check_system_proxy() -> std::result::Result<crate::common::ProxyCheckCode, std::io::Error> {
    println!("检查系统代理状态...");

    // 直接调用 check_clash_process，不通过 clash 命名空间
    let process_running = check_clash_process();
    if !process_running {
        println!("Clash 进程不存在或已停止");
        return Ok(crate::common::ProxyCheckCode::ClashProcessNotRunning);
    }

    #[cfg(target_os = "macos")]
    {
        // 获取HTTP代理状态
        let output = Command::new("networksetup")
            .args(&["-getwebproxy", "Wi-Fi"])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // 检查代理是否启用
        let enabled = output_str.contains("Enabled: Yes");
        if !enabled {
            return Ok(crate::common::ProxyCheckCode::ProxyNotEnabled);
        }
        
        // 检查代理服务器和端口
        let correct_server = output_str.contains("Server: 127.0.0.1") && 
                             output_str.contains(&format!("Port: {}", CLASH_PROXY_PORT));
        
        if !correct_server {
            return Ok(crate::common::ProxyCheckCode::ProxyServerIncorrect);
        }
        
        return Ok(crate::common::ProxyCheckCode::Ok);
    }
    
    #[cfg(target_os = "windows")]
    {
        // 获取Windows系统代理设置
        let reg_query = Command::new("reg")
            .args(&["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", "/v", "ProxyEnable", "/v", "ProxyServer"])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&reg_query.stdout);
        
        // 检查代理是否启用
        let enabled = output_str.contains("ProxyEnable    REG_DWORD    0x1");
        if !enabled {
            return Ok(crate::common::ProxyCheckCode::ProxyNotEnabled);
        }
        
        // 检查代理服务器和端口
        let correct_server = output_str.contains(&format!("ProxyServer    REG_SZ    127.0.0.1:{}", CLASH_PROXY_PORT));
        if !correct_server {
            return Ok(crate::common::ProxyCheckCode::ProxyServerIncorrect);
        }
        
        return Ok(crate::common::ProxyCheckCode::Ok);
    }
    
    #[cfg(target_os = "linux")]
    {
        // 获取GNOME系统代理设置
        let proxy_mode = Command::new("gsettings")
            .args(&["get", "org.gnome.system.proxy", "mode"])
            .output()?;
        
        let host = Command::new("gsettings")
            .args(&["get", "org.gnome.system.proxy.http", "host"])
            .output()?;
        
        let port = Command::new("gsettings")
            .args(&["get", "org.gnome.system.proxy.http", "port"])
            .output()?;
        
        let proxy_mode_str = String::from_utf8_lossy(&proxy_mode.stdout);
        let host_str = String::from_utf8_lossy(&host.stdout);
        let port_str = String::from_utf8_lossy(&port.stdout);
        
        // 检查代理是否启用
        let enabled = proxy_mode_str.trim() == "'manual'";
        if !enabled {
            return Ok(crate::common::ProxyCheckCode::ProxyNotEnabled);
        }
        
        // 检查代理服务器和端口是否正确
        let correct_server = host_str.trim() == "'127.0.0.1'" && 
                             port_str.trim() == &CLASH_PROXY_PORT.to_string();
        if !correct_server {
            return Ok(crate::common::ProxyCheckCode::ProxyServerIncorrect);
        }
        
        return Ok(crate::common::ProxyCheckCode::Ok);
    }
    
    // 默认情况下假设代理配置正确
    #[allow(unreachable_code)]
    Ok(crate::common::ProxyCheckCode::Ok)
}


/// 检查 Clash 进程是否存在并运行
pub fn check_clash_process() -> bool {
    let mut clash_lock = CLASH_PROCESS.lock().unwrap();
    
    if let Some(ref mut child) = *clash_lock {
        // 尝试获取进程状态，如果能获取到退出状态说明进程已结束
        match child.try_wait() {
            Ok(None) => {
                // 进程仍在运行
                println!("Clash 进程正在运行，PID: {:?}", child.id());
                true
            },
            Ok(Some(status)) => {
                // 进程已退出，清除进程引用
                println!("Clash 进程已退出，状态码: {:?}", status.code());
                *clash_lock = None;  // 重要：清除引用
                false
            },
            Err(e) => {
                // 检查进程状态出错，清除可能无效的引用
                println!("检查 Clash 进程状态出错: {:?}", e);
                *clash_lock = None;  // 重要：清除引用
                false
            }
        }
    } else {
        // 没有存储 Clash 进程
        println!("没有找到正在运行的 Clash 进程");
        false
    }
}