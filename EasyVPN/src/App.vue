<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import TrialTimer from "./components/TrialTimer.vue";
import LeftServiceDays from "./components/LeftServiceDays.vue";
import ServiceExpire from "./components/ServiceExpire.vue";
import ErrorMessage from "./components/ErrorMessage.vue";
import Mine from "./Mine.vue";
import Login from "./Login.vue";
import { ErrorMessages, StatusMessages, LogMessages } from './constants/messages';
import { Account } from './types/account';

// 定义响应式状态，使模板能正确识别
defineProps({});

// 页面控制变量
const currentPage = ref('home'); // 可能的值: 'home', 'settings', 'login'

// 用户账号数据
const accountData = ref(null as Account | null);
// 用户状态（试用/付费）- 从账号数据中获取
const userStatus = computed(() => {
  if (!accountData.value) return "NO_SERVICE";
  return accountData.value.status || "NO_SERVICE";
});

// 试用到期时间计算
const trialExpiryDate = computed(() => {
  if (userStatus.value !== "TRIAL" || !accountData.value || !accountData.value.serviceExpiryDate) {
      return null;
  }
  return accountData.value.serviceExpiryDate;
});

// 服务到期时间
const serviceExpiryDate = computed(() => {
  if (userStatus.value !== "ON_SERVICE" || !accountData.value || !accountData.value.serviceExpiryDate) {
    return null;
  }
  return accountData.value.serviceExpiryDate;
});

// 服务月费
const monthlyFee = ref(69.99);

// 连接状态 - 扩展为三种状态
const connectionStatus = ref("disconnected"); // 可能的值: "disconnected", "connected", "failed"
const isLoading = ref(false); // 加载状态
const errorMessage = ref("");

// 跳转到设置页面
function goToSettings() {
  currentPage.value = 'settings';
  logToTerminal('用户进入设置页面');
}

// 跳转到登录页面
function goToLogin() {
  currentPage.value = 'login';
  logToTerminal('用户进入登录页面');
}

// 返回主页
function goToHome() {
  currentPage.value = 'home';
  logToTerminal('用户返回主页');
  // 返回主页时刷新账号状态
  fetchAccountData();
}

// 创建一个自定义日志函数
async function logToTerminal(message: string) {
  await invoke('log_to_console', { message });
  console.log(message); // 仍然在浏览器控制台显示
}

// 获取账号数据
async function fetchAccountData() {
  try {
    const response = await invoke('get_account_info');
    accountData.value = response as Account;
    logToTerminal(LogMessages.DATA_LOADED + " 账号数据: " + JSON.stringify(accountData.value));
  } catch (error) {
    logToTerminal(`获取账号数据失败: ${error}`);
    accountData.value = null;
  }
}

// 连接/断开VPN
async function toggleConnection() {
  logToTerminal("toggleConnection, isLoading: " + isLoading.value);
  if (isLoading.value) return;
  
  // 检查用户状态是否允许连接
  if (connectionStatus.value !== "connected" && 
     (userStatus.value !== "TRIAL" && userStatus.value !== "ON_SERVICE")) {
    connectionStatus.value = "failed";
    errorMessage.value = ErrorMessages.ACCOUNT_STATUS_INVALID;
    logToTerminal(`用户尝试连接VPN失败: 账号状态(${userStatus.value})不允许连接`);
    return;
  }
  
  isLoading.value = true;
  
  try {
    if (connectionStatus.value === "connected") {
      // 如果当前是已连接状态，则调用断开功能
      await invoke('disconnect_vpn');
      connectionStatus.value = "disconnected";
      logToTerminal(LogMessages.VPN_DISCONNECTED);
    } else {
      // 如果当前是未连接或连接失败状态，则调用连接功能
      await invoke('connect_vpn', { restart: true });
      connectionStatus.value = "connected";
      logToTerminal(LogMessages.VPN_CONNECTED);
    }

    logToTerminal(LogMessages.STATUS_CHANGED + ': ' + connectionStatus.value);

  } catch (error) {
    connectionStatus.value = "failed";
    errorMessage.value = ErrorMessages.CONNECTION_FAILED;
    logToTerminal('VPN ' + (connectionStatus.value === "connected" ? '断开' : '连接') + '失败:' + error);
  } finally {
    isLoading.value = false;
  }
}

// 重试连接
function retryConnection() {
  // 重置错误信息
  errorMessage.value = "";
  // 将状态重置为未连接，然后尝试连接
  connectionStatus.value = "disconnected";
  toggleConnection();
}

// 处理购买事件
function handlePurchase() {
  logToTerminal("用户点击了购买按钮");
  // 这里可以调用支付API或跳转到支付页面
}

// 处理登录成功
function handleLoginSuccess(accountInfo: Account) {
  console.log('登录成功，账号信息:', accountInfo);
  // 刷新账号数据
  accountData.value = accountInfo;
  // 返回设置页面
  console.log('准备将当前页面从', currentPage.value, '切换到settings');
  currentPage.value = 'settings';
  console.log('当前页面已切换到', currentPage.value);
  logToTerminal('用户登录成功，返回设置页面');
}

// 处理登录错误
function handleLoginError(error: any) {
  console.error('登录错误:', error);
  logToTerminal(`登录失败: ${error}`);
}

// 组件挂载时初始化
onMounted(async () => {
  try {
    logToTerminal("组件挂载时初始化");
    
    // 获取账号数据
    await fetchAccountData();
    
    // 启动Clash
    await invoke('start_clash');
    
    // 监听后端发出的账号状态变化事件
    const unlistenAccountStatus = await listen('account-status-changed', (event) => {
      logToTerminal('收到账号状态变更事件，当前状态: ' + (accountData.value ? accountData.value.status : 'unknown'));
      accountData.value = event.payload as Account;
      logToTerminal(`收到账号状态变更事件,状态已更新为: ${accountData.value.status} 
      账号数据: ${JSON.stringify(accountData.value)}`);
    });
    
    // 监听从Mine.vue发出的返回事件
    window.addEventListener('go-back', goToHome);
    
    // 监听导航事件
    window.addEventListener('navigate', (event: any) => {
      const { page } = event.detail;
      if (page === 'login') {
        goToLogin();
      } else if (page === 'settings') {
        goToSettings();
      } else if (page === 'home') {
        goToHome();
      }
    });
    
    // 监听登录成功事件
    window.addEventListener('login-success', (event: any) => {
      console.log('收到login-success事件', event.detail);
      const { accountInfo } = event.detail;
      console.log('准备处理登录成功事件，accountInfo:', accountInfo);
      handleLoginSuccess(accountInfo);
      console.log('登录成功事件处理完成，当前页面:', currentPage.value);
    });
    
    // 监听登录失败事件
    window.addEventListener('login-error', (event: any) => {
      const { error } = event.detail;
      handleLoginError(error);
    });
    
    // 组件卸载时清除事件监听
    onUnmounted(() => {
      unlistenAccountStatus(); // 移除事件监听器
      window.removeEventListener('go-back', goToHome);
      window.removeEventListener('navigate', (event: any) => {});
      window.removeEventListener('login-success', (event: any) => {});
      window.removeEventListener('login-error', (event: any) => {});
    });
    
  } catch (error) {
    connectionStatus.value = "failed";
    errorMessage.value = ErrorMessages.CLASH_START_FAILED;
    logToTerminal('初始化失败, 错误详情:' + error);
  }
});

// 组件卸载时清理
onUnmounted(async () => {
  try {
    // 停止Clash并关闭系统代理
    await invoke('stop_clash');
    logToTerminal('Clash已停止并关闭系统代理');
  } catch (error) {
    console.error('停止Clash失败:', error);
  }
});

// 监听用户状态变化，如果用户状态变为不允许连接的状态，则断开连接
watch(userStatus, (newStatus: string, oldStatus: string) => {
  if (connectionStatus.value === "connected" && 
      (newStatus !== "TRIAL" && newStatus !== "ON_SERVICE")) {
    // 用户状态变为不允许连接，自动断开
    logToTerminal(`用户状态变更(${oldStatus} -> ${newStatus})，自动断开VPN连接`);
    connectionStatus.value = "failed";
    errorMessage.value = ErrorMessages.ACCOUNT_CHANGED;
  }
});

// 将需要在模板中使用的变量和方法定义为组件属性
defineExpose({
  userStatus,
  connectionStatus,
  toggleConnection,
  retryConnection,
  isLoading,
  trialExpiryDate,
  serviceExpiryDate,
  monthlyFee,
  handlePurchase,
  errorMessage,
  currentPage,
  goToSettings,
  goToHome
});
</script>

<template>
  <!-- 根据currentPage显示不同页面 -->
  <div v-if="currentPage === 'home'">
    <!-- Main Content -->
    <main id="main-content" class="max-w-md mx-auto px-3 py-3">
      <!-- Status Card -->
      <div id="status-card" class="bg-white rounded-2xl shadow-lg p-8 text-center">
        <!-- Logo -->
        <div id="app-logo" class="flex items-center justify-between mb-8">
          <div class="flex items-center space-x-2">
            <i class="fa-solid fa-shield-halved text-pink-500 text-3xl"></i>
            <h1 class="text-2xl font-semibold text-gray-800">EasyVPN</h1>
          </div>
          <button id="settings-button" class="text-gray-400 hover:text-pink-500 transition-colors duration-200" @click="goToSettings">
            <i class="fa-solid fa-bars text-xl"></i>
          </button>
        </div>
        
        <!-- 未连接状态 -->
        <div v-if="connectionStatus === 'disconnected'" id="connection-status" class="mb-8">
          <div class="w-32 h-32 mx-auto mb-6 relative">
            <div class="absolute inset-0 rounded-full bg-pink-100 animate-pulse"></div>
            <div class="absolute inset-3 rounded-full bg-pink-200"></div>
            <div class="absolute inset-6 rounded-full bg-pink-300"></div>
            <div class="absolute inset-9 rounded-full bg-pink-400 flex items-center justify-center">
              <i class="fa-solid fa-power-off text-white text-2xl"></i>
            </div>
          </div>
          <h2 class="text-xl font-semibold text-gray-800 mb-2">{{ StatusMessages.DISCONNECTED }}</h2>
        </div>
        
        <!-- 已连接状态 -->
        <div v-if="connectionStatus === 'connected'" id="connection-status" class="mb-8">
          <div class="w-32 h-32 mx-auto mb-6 relative">
            <div class="absolute inset-0 rounded-full bg-green-100 animate-pulse"></div>
            <div class="absolute inset-3 rounded-full bg-green-200"></div>
            <div class="absolute inset-6 rounded-full bg-green-300"></div>
            <div class="absolute inset-9 rounded-full bg-green-400 flex items-center justify-center">
              <i class="fa-solid fa-power-off text-white text-2xl"></i>
            </div>
          </div>
          <h2 class="text-xl font-semibold text-gray-800 mb-2">{{ StatusMessages.CONNECTED }}</h2>
        </div>
        
        <!-- 连接失败状态 -->
        <div v-if="connectionStatus === 'failed'" id="connection-status" class="mb-4">
          <div class="w-32 h-32 mx-auto mb-4 relative">
            <div class="absolute inset-0 rounded-full bg-red-100"></div>
            <div class="absolute inset-3 rounded-full bg-red-200"></div>
            <div class="absolute inset-6 rounded-full bg-red-300"></div>
            <div class="absolute inset-9 rounded-full bg-red-400 flex items-center justify-center">
              <i class="fa-solid fa-xmark text-white text-2xl"></i>
            </div>
          </div>
          <h2 class="text-xl font-semibold text-gray-800 mb-1">{{ StatusMessages.FAILED }}</h2>
        </div>

        <!-- 错误信息组件 (仅在连接失败时显示) -->
        <ErrorMessage v-if="connectionStatus === 'failed'" :message="errorMessage" />

        <!-- 连接/断开按钮 - 根据状态显示不同按钮 -->
        <button 
          v-if="connectionStatus !== 'failed'"
          id="connect-button" 
          class="w-full text-white rounded-lg py-4 mb-6 transition-colors duration-200 flex items-center justify-center space-x-2"
          :class="connectionStatus === 'connected' ? 'bg-green-500 hover:bg-green-600' : 'bg-pink-500 hover:bg-pink-600'"
          @click="toggleConnection"
          :disabled="isLoading || (connectionStatus === 'disconnected' && userStatus !== 'TRIAL' && userStatus !== 'ON_SERVICE')"
        >
          <i v-if="isLoading" class="fa-solid fa-spinner fa-spin"></i>
          <i v-else :class="connectionStatus === 'connected' ? 'fa-solid fa-plug-circle-xmark' : 'fa-solid fa-plug-circle-bolt'"></i>
          <span>{{ connectionStatus === 'connected' ? '断开' : '连接' }}</span>
        </button>
        
        <!-- 重试按钮 - 仅在连接失败时显示 -->
        <button 
          v-if="connectionStatus === 'failed'"
          id="connect-button" 
          class="w-full bg-pink-500 hover:bg-pink-600 text-white rounded-lg py-3 mb-6 transition-colors duration-200 flex items-center justify-center space-x-2"
          @click="retryConnection"
          :disabled="isLoading"
        >
          <i v-if="isLoading" class="fa-solid fa-spinner fa-spin"></i>
          <i v-else class="fa-solid fa-rotate"></i>
          <span>重试</span>
        </button>
        
        <!-- 根据用户状态显示不同的组件 -->
        <TrialTimer 
          v-if="userStatus === 'TRIAL'" 
          :expiryDate="trialExpiryDate" 
        />
        <LeftServiceDays 
          v-if="userStatus === 'ON_SERVICE'" 
          :expiryDate="serviceExpiryDate" 
        />
        <ServiceExpire 
          v-if="userStatus === 'TRIAL_END' || userStatus === 'SERVICE_END' || userStatus === 'NO_SERVICE'" 
          :price="monthlyFee"
          @purchase="handlePurchase"
        />
      </div>
    </main>
  </div>

  <!-- 设置页面 -->
  <Mine v-else-if="currentPage === 'settings'" />

  <!-- 登录页面 -->
  <Login v-else-if="currentPage === 'login'" />
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  color: #0f0f0f;
  background-color: #f6f6f6;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

/* 暗黑模式 */
@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
}
</style> 