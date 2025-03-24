<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core';
import TrialTimer from "./components/TrialTimer.vue";
import LeftServiceDays from "./components/LeftServiceDays.vue";
import ServiceExpire from "./components/ServiceExpire.vue";

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

// 连接状态 - 简化后的状态
const isConnected = ref(false);
const isLoading = ref(false); // 加载状态

// 连接/断开VPN
async function toggleConnection() {
  isLoading.value = true;
  
  // 模拟网络延迟
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  // 切换连接状态
  isConnected.value = !isConnected.value;
  
  isLoading.value = false;
  
  // 打印日志以便调试
  console.log(`VPN 连接状态已切换为: ${isConnected.value ? '已连接' : '未连接'}`);
}

// 处理购买事件
function handlePurchase() {
  console.log("用户点击了购买按钮");
  // 这里可以调用支付API或跳转到支付页面
}

// 组件挂载时初始化
onMounted(async () => {
  console.log("应用程序已启动");
  
  // 只保留问候测试
  try {
    const message = await invoke('greet', { name: '用户' });
    console.log(message);
  } catch (error) {
    console.error('调用 greet 失败:', error);
  }
});

// 将需要在模板中使用的变量和方法定义为组件属性
defineExpose({
  userStatus,
  isConnected,
  toggleConnection,
  isLoading,
  totalTrialSeconds,
  serviceDaysLeft,
  monthlyFee,
  handlePurchase
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
      
      <!-- Connection Status -->
      <div id="connection-status" class="mb-8">
        <div class="w-32 h-32 mx-auto mb-6 relative">
          <div 
            class="absolute inset-0 rounded-full" 
            :class="isConnected ? 'bg-green-100 animate-pulse' : 'bg-pink-100 animate-pulse'"
          ></div>
          <div 
            class="absolute inset-3 rounded-full" 
            :class="isConnected ? 'bg-green-200' : 'bg-pink-200'"
          ></div>
          <div 
            class="absolute inset-6 rounded-full" 
            :class="isConnected ? 'bg-green-300' : 'bg-pink-300'"
          ></div>
          <div 
            class="absolute inset-9 rounded-full flex items-center justify-center" 
            :class="isConnected ? 'bg-green-400' : 'bg-pink-400'"
          >
            <i class="fa-solid fa-power-off text-white text-2xl"></i>
          </div>
        </div>
        <h2 class="text-xl font-semibold text-gray-800 mb-2">
          {{ isConnected ? '已连接' : '未连接' }}
        </h2>
        <p class="text-gray-500 text-sm">
          {{ isConnected ? '您已连接到安全VPN' : '点击连接到安全VPN' }}
        </p>
      </div>

      <!-- Connect Button -->
      <button 
        id="connect-button" 
        class="w-full text-white rounded-lg py-4 mb-6 transition-colors duration-200 flex items-center justify-center space-x-2"
        :class="isConnected ? 'bg-green-500 hover:bg-green-600' : 'bg-pink-500 hover:bg-pink-600'"
        @click="toggleConnection"
        :disabled="isLoading"
      >
        <i v-if="isLoading" class="fa-solid fa-spinner fa-spin"></i>
        <i v-else :class="isConnected ? 'fa-solid fa-plug-circle-xmark' : 'fa-solid fa-plug-circle-bolt'"></i>
        <span>{{ isConnected ? '断开' : '连接' }}</span>
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