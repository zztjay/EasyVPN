use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyCheckCode {
    Ok = 0,
    ClashProcessNotRunning = 1,
    ProxyNotEnabled = 2,
    ProxyServerIncorrect = 3,
    CheckError = 4,
}

impl ProxyCheckCode {
    pub fn get_message(&self) -> &'static str {
        match self {
            Self::Ok => "系统代理运行正常",
            Self::ClashProcessNotRunning => "Clash进程未运行，请重新启动应用",
            Self::ProxyNotEnabled => "系统代理未启用，请重新连接",
            Self::ProxyServerIncorrect => "系统代理配置错误，请重新连接",
            Self::CheckError => "系统代理检查失败，请检查网络连接",
        }
    }
}