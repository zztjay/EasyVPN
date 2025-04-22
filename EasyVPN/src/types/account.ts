// 设备信息接口
export interface DeviceInfo {
  id?: number;            // 前端生成的ID
  deviceId?: number;      // 后端设备ID
  name?: string;          // 前端显示用名称
  deviceName?: string | null;  // 后端设备名称
  type?: string;          // 前端显示用类型
  lastActive?: string;    // 前端显示用最后活动时间
  lastOnlineTime?: string; // 后端最后在线时间
  deviceUserId: number;   // 设备用户ID (整数类型)
  macAddress?: string;    // MAC地址
  online?: boolean;       // 前端显示用在线状态
  active?: boolean;       // 后端设备活跃状态
  trialExpiryDate?: string | null; // 试用过期时间
}

// 账号信息接口 - 与后端 account.rs 中的 Account 结构保持一致
export interface Account {
  accountId: string;
  accessToken: string;
  refreshToken: string;
  status: string;
  guest: boolean;
  serviceExpiryDate?: string;
  userId?: number;
  username?: string;
  devices?: DeviceInfo[];
  loginType?: string;     // 登录类型
} 