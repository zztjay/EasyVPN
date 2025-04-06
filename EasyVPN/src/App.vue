<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core';
import TrialTimer from "./components/TrialTimer.vue";
import LeftServiceDays from "./components/LeftServiceDays.vue";
import ServiceExpire from "./components/ServiceExpire.vue";
import ErrorMessage from "./components/ErrorMessage.vue";

// 定义响应式状态，使模板能正确识别
defineProps({});

// 用户状态（试用/付费）
const userStatus = ref("serviceExpire"); // 可能的值: "trial", "paid", "trialEnd", "serviceExpire"

// 试用期总秒数 (7天12小时30分钟 = 652200秒)
const totalTrialSeconds = ref(652200);

// 付费用户的剩余服务天数
const serviceDaysLeft = ref(27);

// 服务月费
const monthlyFee = ref(69.99);

// 连接状态 - 扩展为三种状态
const connectionStatus = ref("disconnected"); // 可能的值: "disconnected", "connected", "failed"
const isLoading = ref(false); // 加载状态
const errorMessage = ref("服务器无响应，请检查网络连接后重试");

// 创建一个自定义日志函数
async function logToTerminal(message: string) {
  await invoke('log_to_console', { message });
  console.log(message); // 仍然在浏览器控制台显示
}

// 连接/断开VPN
async function toggleConnection() {
  if (isLoading.value) return;
  
  isLoading.value = true;
  
  try {
    if (connectionStatus.value === "connected") {
      // 如果当前是已连接状态，则调用断开功能
      await invoke('disconnect_vpn');
      connectionStatus.value = "disconnected";
      logToTerminal('VPN已断开连接（Direct模式）');
    } else {
      // 如果当前是未连接或连接失败状态，则调用连接功能
      await invoke('connect_vpn', { restart: true });
      connectionStatus.value = "connected";
      logToTerminal('VPN已连接（Rule模式）');
    }

    logToTerminal('连接状态已切换为:' + connectionStatus.value);

  } catch (error) {
    connectionStatus.value = "failed";
    errorMessage.value = String(error) || "连接失败，请稍后重试";
    logToTerminal('VPN ' + (connectionStatus.value === "connected" ? '断开' : '连接') + '失败:' + error);
  } finally {
    isLoading.value = false;
  }
}

// 重试连接
function retryConnection() {
  // 将状态重置为未连接，然后尝试连接
  connectionStatus.value = "disconnected";
  toggleConnection();
}

// 处理购买事件
function handlePurchase() {
  console.log("用户点击了购买按钮");
  // 这里可以调用支付API或跳转到支付页面
}


// 组件挂载时初始化
onMounted(async () => {
  
  try {
    // 启动Clash
    await invoke('start_clash');

    // 设置定时器，每6秒检查一次系统代理状态
    const proxyCheckInterval = setInterval(async () => {
      try {
        if(connectionStatus.value === "connected"){
          const proxyCheckResult = await invoke('check_system_proxy');
          if (proxyCheckResult!= 'Ok') {
            connectionStatus.value = "failed";
            errorMessage.value = "系统代理异常，请重新连接！";
            logToTerminal('系统代理检查失败，VPN连接状态已更新为失败');
          }
      }
      } catch (error) {
        logToTerminal('检查系统代理异常:' + error);
      }
    }, 6000); 
    
    // 组件卸载时清除定时器
    onUnmounted(() => {
      clearInterval(proxyCheckInterval);
    });
    
  } catch (error) {
    connectionStatus.value = "failed";
    errorMessage.value = "EasyVPN启动失败，请重启应用！";
    logToTerminal('初始化Clash失败, 错误详情:' + error);
  }
  
});

// 组件卸载时清理
onUnmounted(async () => {
  try {
    // 停止Clash并关闭系统代理
    await invoke('stop_clash');
    console.log('Clash已停止并关闭系统代理');
  } catch (error) {
    console.error('停止Clash失败:', error);
  }
});

// 将需要在模板中使用的变量和方法定义为组件属性
defineExpose({
  userStatus,
  connectionStatus,
  toggleConnection,
  retryConnection,
  isLoading,
  totalTrialSeconds,
  serviceDaysLeft,
  monthlyFee,
  handlePurchase,
  errorMessage
});
</script>

<template>
  <!-- Main Content -->
  <main id="main-content" class="max-w-md mx-auto px-3 py-3">
    <!-- Status Card -->
    <div id="status-card" class="bg-white rounded-2xl shadow-lg p-8 text-center">
      <!-- Logo -->
      <div id="app-logo" class="flex items-center space-x-2 mb-8">
        <i class="fa-solid fa-shield-halved text-pink-500 text-3xl"></i>
        <h1 class="text-2xl font-semibold text-gray-800">EasyVPN</h1>
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
        <h2 class="text-xl font-semibold text-gray-800 mb-2">未连接</h2>
        <!-- <p class="text-gray-500 text-sm">点击连接到安全VPN</p> -->
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
        <h2 class="text-xl font-semibold text-gray-800 mb-2">已连接</h2>
        <!-- <p class="text-gray-500 text-sm">您已连接到安全VPN</p> -->
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
        <h2 class="text-xl font-semibold text-gray-800 mb-1">连接失败</h2>
        <!-- <p class="text-gray-500 text-sm">无法建立VPN连接</p> -->
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
        :disabled="isLoading"
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
      <TrialTimer v-if="userStatus === 'trial'" :total-seconds="totalTrialSeconds" />
      <LeftServiceDays v-if="userStatus === 'paid'" :service-days="serviceDaysLeft" />
      <ServiceExpire 
        v-if="userStatus === 'trialEnd' || userStatus === 'serviceExpire'" 
        :price="monthlyFee"
        @purchase="handlePurchase"
      />
    </div>
  </main>
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