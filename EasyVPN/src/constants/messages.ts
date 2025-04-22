/**
 * 错误提示文案
 */
export const ErrorMessages = {
  // 连接相关错误
  CONNECTION_FAILED: "连接失败，请重试",
  PROXY_ERROR: "系统代理异常",
  CLASH_START_FAILED: "启动失败，请重启",
  
  // 账号状态相关错误
  ACCOUNT_STATUS_INVALID: "账号状态无效",
  SERVICE_EXPIRED: "服务已到期",
  ACCOUNT_CHANGED: "账号状态已变更",
  
  // 通用错误
  DATA_LOAD_FAILED: "数据加载失败",
  SERVER_NO_RESPONSE: "服务器无响应"
};

/**
 * 操作状态文案
 */
export const StatusMessages = {
  CONNECTED: "已连接",
  DISCONNECTED: "未连接",
  CONNECTING: "连接中...",
  FAILED: "连接失败"
};

/**
 * 账号状态文案
 */
export const AccountMessages = {
  TRIAL: "试用中",
  ON_SERVICE: "服务中",
  SERVICE_END: "服务已到期",
  TRIAL_END: "试用已结束",
  NO_SERVICE: "未购买服务"
};

/**
 * 日志信息
 */
export const LogMessages = {
  VPN_CONNECTED: "VPN已连接(Rule模式)",
  VPN_DISCONNECTED: "VPN已断开(Direct模式)",
  STATUS_CHANGED: "连接状态已切换",
  PROXY_CHECK_FAILED: "代理检查失败",
  DATA_LOADED: "账号数据已加载"
}; 