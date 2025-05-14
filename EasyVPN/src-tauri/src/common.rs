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

// 定义账号状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    NO_SERVICE,    // 未购买服务（普通账号）
    ON_SERVICE,    // 服务中（普通账号）
    SERVICE_END,   // 服务到期（普通账号）
    TRIAL,         // 试用期中（设备游客账号）
    TRIAL_END,     // 试用期结束（设备游客账号）
    NO_INIT // 未初始化
}

impl AccountStatus {
    pub fn get_message(&self) -> &'static str {
        match self {
            Self::NO_SERVICE => "未购买服务",
            Self::ON_SERVICE => "服务中",
            Self::SERVICE_END => "服务已到期",
            Self::TRIAL => "试用期中",
            Self::TRIAL_END => "试用期已结束",
            Self::NO_INIT => "未初始化",
        }
    }
    
    // 从字符串转换为枚举
    pub fn from_str(status: &str) -> Self {
        match status {
            "NO_SERVICE" => Self::NO_SERVICE,
            "ON_SERVICE" => Self::ON_SERVICE,
            "SERVICE_END" => Self::SERVICE_END,
            "TRIAL" => Self::TRIAL,
            "TRIAL_END" => Self::TRIAL_END,
            "NO_INIT" => Self::NO_INIT,
            _ => Self::NO_INIT, // 默认为未初始化状态
        }
    }
    
    // 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NO_SERVICE => "NO_SERVICE",
            Self::ON_SERVICE => "ON_SERVICE",
            Self::SERVICE_END => "SERVICE_END",
            Self::TRIAL => "TRIAL",
            Self::TRIAL_END => "TRIAL_END",
            Self::NO_INIT => "NO_INIT",
        }
    }
}