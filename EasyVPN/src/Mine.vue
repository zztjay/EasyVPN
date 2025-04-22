<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Account } from './types/account';

// 定义响应式数据
const accountId = ref("VPN285719");
const userName = ref("张伟"); // 添加用户名
const userAvatar = ref("https://storage.googleapis.com/uxpilot-auth.appspot.com/avatars/avatar-2.jpg"); // 添加用户头像
const userStatus = ref("SERVICE_END"); // 可能的值: "TRIAL", "ON_SERVICE", "TRIAL_END", "SERVICE_END", "NO_SERVICE"
const isLoggedIn = ref(false); // 添加登录状态
const startDate = ref("2025-01-01");
const endDate = ref("2025-01-14");
const remainingInvites = ref(5);
const inviteCode = ref("FRIEND25XP");
const deviceCount = ref(3);
const maxDevices = ref(5);
const devices = ref([
  {
    id: 1,
    name: "MacBook Pro",
    type: "laptop",
    lastActive: "今天",
    online: true,
    deviceUserId: "device_mac_001"
  },
  {
    id: 2,
    name: "iPhone 14 Pro",
    type: "mobile-screen",
    lastActive: "2小时前",
    online: false,
    deviceUserId: "device_iphone_002"
  },
  {
    id: 3,
    name: "iPad Air",
    type: "tablet",
    lastActive: "昨天",
    online: false,
    deviceUserId: "device_ipad_003"
  }
]);

// 添加解绑状态
const isUnbinding = ref(false);
const unbindError = ref('');

// 事件处理函数
function goBack() {
  // 触发返回事件，由父组件处理
  const event = new CustomEvent('go-back');
  window.dispatchEvent(event);
}

function copyInviteCode() {
  navigator.clipboard.writeText(inviteCode.value)
    .then(() => {
      // 可以添加一个提示，表示复制成功
      console.log("邀请码已复制");
    })
    .catch(err => {
      console.error("复制失败:", err);
    });
}

async function removeDevice(deviceId: number, deviceUserId: number) {
  // 确认对话框
  if (!confirm('确定要解绑该设备吗？')) {
    return;
  }
  
  try {
    // 设置解绑中状态
    isUnbinding.value = true;
    unbindError.value = '';
    
    // 获取当前设备ID（从设备列表中找到在线的设备作为当前设备）
    let currentDeviceUserId: number | null = null;
    const currentDevice = devices.value.find((d: any) => d.online === true);
    if (currentDevice) {
      currentDeviceUserId = currentDevice.deviceUserId;
    }
    
    // 调用后端API解绑设备，传入当前设备ID以便解绑后激活它
    const result = await invoke('unbind_device', { 
      deviceUserId: deviceUserId.toString(), // 转换为字符串传递给后端
      currentDeviceUserId: currentDeviceUserId ? currentDeviceUserId.toString() : null
    });
    console.log('解绑设备结果:', result);
    
    // 本地更新设备列表
    devices.value = devices.value.filter((device: {id: number}) => device.id !== deviceId);
    deviceCount.value -= 1;
    
  } catch (error) {
    console.error("解绑设备失败:", error);
    unbindError.value = String(error);
  } finally {
    isUnbinding.value = false;
  }
}

function handlePurchase() {
  console.log("用户点击了购买按钮");
  // 调用支付API或跳转到支付页面
}

// 跳转到登录页面
function handleLogin() {
  console.log("用户点击了登录按钮");
  // 通知App.vue切换到登录页面
  const event = new CustomEvent('navigate', { 
    detail: { page: 'login' }
  });
  window.dispatchEvent(event);
}

// 处理登录成功 - 将由App.vue中的事件处理
function handleLoginSuccess(accountInfo: Account) {
  console.log('登录成功，账号信息:', accountInfo);
  // 更新登录状态
  isLoggedIn.value = true;
  if (accountInfo.username) {
    userName.value = accountInfo.username;
  }
  // 刷新账号信息
  refreshAccountInfo();
}

// 处理登录错误
function handleLoginError(error: any) {
  console.error('登录错误:', error);
}

// 刷新账号信息
async function refreshAccountInfo() {
  try {
    // 获取账号信息
    const accountInfo = await invoke('get_account_info') as Account;
    console.log("获取到账号信息:", accountInfo);

    // 更新账号数据
    if (accountInfo) {
      // 这里可以根据返回的数据更新本地状态
      accountId.value = accountInfo.accountId || accountId.value;
      userStatus.value = accountInfo.status || userStatus.value;
      
      // 更新登录状态和用户名
      if (accountInfo.username && accountInfo.accessToken && accountInfo.accessToken.trim() !== '') {
        userName.value = accountInfo.username;
        isLoggedIn.value = true;
        
        // 如果是特定测试账号，使用预设头像
        if (accountInfo.username === 'zztjay') {
          userName.value = '张伟'; // 使用中文名
          userAvatar.value = 'https://storage.googleapis.com/uxpilot-auth.appspot.com/avatars/avatar-2.jpg';
        }
      } else {
        isLoggedIn.value = false;
        userName.value = "未登录用户";
      }
      
      if (accountInfo.serviceExpiryDate) {
        endDate.value = accountInfo.serviceExpiryDate;
      }
      
      if (accountInfo.devices) {
        // 转换设备格式以适配界面显示需求
        devices.value = accountInfo.devices.map((device: any, index: number) => {
          return {
            id: index + 1,
            name: device.deviceName || `设备 ${index + 1}`,
            type: device.deviceName?.toLowerCase().includes("iphone") ? "mobile-screen" : 
                  device.deviceName?.toLowerCase().includes("ipad") ? "tablet" : "laptop",
            lastActive: device.lastOnlineTime ? new Date(device.lastOnlineTime).toLocaleDateString() : "未知",
            online: device.active || false,
            deviceUserId: device.deviceUserId
          };
        });
        deviceCount.value = accountInfo.devices.length;
      } else {
        // 如果没有设备信息，清空列表
        devices.value = [];
        deviceCount.value = 0;
      }
      
      console.log("账号状态更新完成，登录状态:", isLoggedIn.value, "用户:", userName.value);
    } else {
      // 如果没有账号信息，则认为未登录
      isLoggedIn.value = false;
      userName.value = "未登录用户";
      console.log("未获取到账号信息，设置为未登录状态");
    }
  } catch (error) {
    console.error("获取账号信息失败:", error);
    isLoggedIn.value = false;
    userName.value = "未登录用户";
  }
}

// 添加登出功能
async function handleLogout() {
  try {
    console.log("正在退出登录...");
    const result = await invoke('logout');
    console.log("退出登录成功:", result);
    
    // 重置登录状态
    isLoggedIn.value = false;
    userName.value = "";
    // 刷新账号信息
    refreshAccountInfo();
    
    // 显示成功提示
    alert("已成功退出登录");
  } catch (error) {
    console.error("退出登录失败:", error);
    
    // 显示错误消息
    alert(`退出登录失败: ${error}`);
    
    // 尝试刷新账号信息，因为可能发生了部分状态变化
    refreshAccountInfo();
  }
}

// 在组件挂载时获取用户账号信息
onMounted(async () => {
  try {
    // 获取账号信息
    await refreshAccountInfo();
    
    // 监听登录成功事件
    window.addEventListener('login-success', (event: any) => {
      const { accountInfo } = event.detail;
      handleLoginSuccess(accountInfo);
    });
  } catch (error) {
    console.error("获取账号信息失败:", error);
  }
  
  // 组件卸载时清理
  return () => {
    window.removeEventListener('login-success', (event: any) => {});
  };
});
</script>

<template>
       <!-- Settings Header -->
       <div id="settings-header" class="flex items-center px-3 py-3">
        <button class="mr-3" @click="goBack">
          <i class="fa-solid fa-arrow-left text-gray-600"></i>
        </button>
        <h1 class="text-xl font-bold text-gray-800">设置</h1>
      </div>
  <!-- Main Content -->
  <main class="pb-6 px-3 ">
    <div class="max-w-[400px] mx-auto h-[600px] overflow-y-auto scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-gray-100">

      <!-- Account Information & Status Combined -->
      <section id="account-info" class="bg-white rounded-lg shadow-sm p-4 mb-4">
        <!-- 登录成功显示账号信息 -->
        <div v-if="isLoggedIn" class="flex items-center space-x-3 mb-4">
          <img :src="userAvatar" alt="Profile" class="w-12 h-12 rounded-full"/>
          <div>
            <p class="font-medium">{{ userName }}</p>
            <p class="text-gray-500 text-xs">ID: {{ accountId }}</p>
          </div>
        </div>
        <!-- 未登录显示默认信息 -->
        <div v-else class="flex items-center space-x-3 mb-4">
          <i class="fa-regular fa-user-circle text-gray-400 text-4xl"></i>
          <div>
            <p class="font-medium">未登录账号</p>
            <p class="text-gray-500 text-xs">ID: {{ accountId }}</p>
          </div>
        </div>
        
        <!-- 根据用户状态显示不同的状态信息 -->
        <!-- 服务已到期 -->
        <div v-if="userStatus === 'SERVICE_END' || userStatus === 'TRIAL_END' || userStatus === 'NO_SERVICE'" 
             class="flex items-center justify-between bg-red-50 p-3 rounded-lg mb-3">
          <div class="flex items-center space-x-2">
            <i class="fa-solid fa-circle-exclamation text-red-500"></i>
            <span class="text-sm font-medium text-red-600">服务已到期</span>
          </div>
          <span class="text-sm text-red-600">已过期</span>
        </div>
        
        <!-- 试用中 -->
        <div v-else-if="userStatus === 'TRIAL'" 
             class="flex items-center justify-between bg-blue-50 p-3 rounded-lg mb-3">
          <div class="flex items-center space-x-2">
            <i class="fa-solid fa-clock text-blue-500"></i>
            <span class="text-sm font-medium text-blue-600">试用中</span>
          </div>
          <span class="text-sm text-blue-600">{{ endDate }}</span>
        </div>
        
        <!-- 正常服务中 -->
        <div v-else-if="userStatus === 'ON_SERVICE'" 
             class="flex items-center justify-between bg-green-50 p-3 rounded-lg mb-3">
          <div class="flex items-center space-x-2">
            <i class="fa-solid fa-check-circle text-green-500"></i>
            <span class="text-sm font-medium text-green-600">服务中</span>
          </div>
          <span class="text-sm text-green-600">{{ endDate }}</span>
        </div>
        
        <div class="space-y-2 text-sm mb-4">
          <div class="flex justify-between text-gray-600">
            <span>开始时间</span>
            <span>{{ startDate }}</span>
          </div>
          <div class="flex justify-between text-gray-600">
            <span>到期时间</span>
            <span>{{ endDate }}</span>
          </div>
        </div>
        
        <!-- 根据登录状态显示不同的操作区域 -->
        <div v-if="!isLoggedIn" class="space-y-3">
          <div class="bg-gray-50 p-3 rounded-lg">
            <p class="text-sm text-gray-600">登录账号后可以直接使用已购买的会员时长</p>
          </div>
          <button class="w-full flex items-center justify-center p-2.5 bg-pink-500 text-white rounded-lg hover:bg-pink-600" @click="handlePurchase">
            <span class="text-sm">购买续期</span>
          </button>
          <button class="w-full flex items-center justify-between p-2.5 border rounded-lg hover:bg-gray-50" @click="handleLogin">
            <span class="text-sm">登录账号</span>
            <i class="fa-solid fa-chevron-right text-gray-400 text-xs"></i>
          </button>
        </div>
        
        <!-- 已登录用户的操作区域 -->
        <div v-else class="space-y-3">
          <button class="w-full flex items-center justify-center p-2.5 bg-pink-500 text-white rounded-lg hover:bg-pink-600" @click="handlePurchase">
            <span class="text-sm">购买续期</span>
          </button>
          <button class="w-full flex items-center justify-between p-2.5 border rounded-lg hover:bg-gray-50" @click="handleLogout">
            <span class="text-sm">退出登录</span>
            <i class="fa-solid fa-sign-out-alt text-gray-400 text-xs"></i>
          </button>
        </div>
      </section>

      <!-- Language Selection -->
      <section id="language-settings" class="bg-white rounded-lg shadow-sm p-4 mb-4">
        <h2 class="text-base font-semibold mb-3">语言</h2>
        <select class="w-full p-2 border rounded-lg bg-gray-50 text-sm">
          <option>中文（简体）</option>
          <option>English (US)</option>
          <option>日本語</option>
          <option>한국어</option>
        </select>
      </section>

      <!-- Referral Section -->
      <section id="referral-section" class="bg-white rounded-lg shadow-sm p-4 mb-4">
        <div class="flex justify-between items-start">
          <div>
            <h2 class="text-base font-semibold mb-1">邀请好友</h2>
            <p class="text-gray-500 text-xs mb-3">每邀请一位好友加入可获得1个月免费服务</p>
          </div>
          <span class="bg-pink-100 text-pink-600 px-2 py-1 rounded-full text-xs">剩余{{ remainingInvites }}次邀请</span>
        </div>
        <div class="bg-gray-50 p-3 rounded-lg flex items-center justify-between">
          <code class="text-xs text-gray-600">{{ inviteCode }}</code>
          <button class="text-pink-500 hover:text-pink-600" @click="copyInviteCode">
            <i class="fa-regular fa-copy"></i>
          </button>
        </div>
      </section>

      <!-- Support Section -->
      <section id="support-section" class="bg-white rounded-lg shadow-sm p-4 mb-4">
        <h2 class="text-base font-semibold mb-3">客户支持</h2>
        <button class="w-full flex items-center justify-center space-x-2 p-3 border rounded-lg hover:bg-gray-50">
          <i class="fa-solid fa-message text-pink-500 text-sm"></i>
          <span class="text-sm">在线客服</span>
        </button>
      </section>

      <!-- Bound Devices -->
      <section id="devices-section" class="bg-white rounded-lg shadow-sm p-4 mb-4">
        <div class="flex justify-between items-center mb-3">
          <h2 class="text-base font-semibold">已绑定设备</h2>
          <span class="text-xs text-gray-500">{{ deviceCount }}/{{ maxDevices }} 台设备</span>
        </div>
        <div class="space-y-3">
          <div v-for="device in devices" :key="device.id" class="flex items-center justify-between p-3 border rounded-lg">
            <div class="flex items-center space-x-3">
              <i :class="`fa-solid fa-${device.type} text-gray-400`"></i>
              <div>
                <p class="font-medium text-sm">{{ device.name }}</p>
                <p class="text-xs text-gray-500">最后活动：{{ device.lastActive }}</p>
              </div>
            </div>
            <div class="flex items-center space-x-2">
              <span class="text-xs" :class="device.online ? 'text-green-500' : 'text-gray-500'">
                {{ device.online ? '在线' : '离线' }}
              </span>
              <button class="text-red-500 hover:text-red-600" @click="removeDevice(device.id, device.deviceUserId)">
                <i class="fa-solid fa-trash-can text-sm"></i>
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
/* 可以添加特定于Mine.vue的样式 */
</style>