// 设备信息接口
export interface DeviceInfo {
  deviceId?: string | null;     // 设备ID
  deviceName?: string | null;  // 设备名称
  deviceUserId: number;        // 设备用户ID (整数类型)
  lastOnlineTime: string;      // 最后在线时间
  trialExpiryDate?: string | null; // 试用过期时间
}

// 账号信息接口 - 与后端 account.rs 中的 Account 结构保持一致
export interface Account {
  accessToken: string;         // 访问令牌
  refreshToken: string;        // 刷新令牌
  status: string;              // 账号状态
  serviceExpiryDate?: string;  // 服务到期时间
  username?: string;           // 用户名
  devices?: DeviceInfo[];      // 设备信息
  loginType?: string;          // 登录类型 ('User'或'Device')
  maxDevicesAllowed?: number;  // 最大设备数
  currentPackageType?: string; // 当前套餐类型
  remainingDays?: number;      // 剩余天数
}