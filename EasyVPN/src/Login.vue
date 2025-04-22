<script setup lang="ts">
import { ref, defineEmits } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Account } from './types/account';

// 定义事件
const emit = defineEmits(['close', 'login-success', 'login-error']);

// 响应式状态数据
const username = ref('zztjay');
const password = ref('jayjay');
const isPasswordVisible = ref(false);
const isLoading = ref(false);
const errorMessage = ref('');

// 登录函数
async function handleLogin() {
  // 表单验证
  if (!username.value.trim()) {
    errorMessage.value = '请输入账号';
    return;
  }
  if (!password.value.trim()) {
    errorMessage.value = '请输入密码';
    return;
  }

  try {
    // 设置加载状态
    isLoading.value = true;
    errorMessage.value = '';
    console.log('准备发起登录请求');
    
    // 调用Rust后端登录接口 - 修改deviceId为deviceUserId
    const result = await invoke('login', {
      auth: {
        username: username.value,
        password: password.value,
        deviceUserId: null // 后端会自动获取设备ID，参数名从deviceId改为deviceUserId
      }
    }) as string;
    
    // 解析返回的账号信息
    const accountInfo = JSON.parse(result) as Account;
    console.log('登录成功:', accountInfo);
    
    // 通知App.vue登录成功（保留原有事件兼容）
    const loginSuccessEvent = new CustomEvent('login-success', {
      detail: { accountInfo }
    });
    window.dispatchEvent(loginSuccessEvent);
    console.log('已发送登录成功事件');
    
    // 返回设置页面
    const navigateEvent = new CustomEvent('navigate', { 
      detail: { page: 'settings' }
    });
    window.dispatchEvent(navigateEvent);
    console.log('已发送导航事件，期望导航到settings页面');
    
  } catch (error) {
    // 处理登录失败
    console.error('登录失败:', error);
    errorMessage.value = String(error) || '登录失败，请检查账号密码';
    // 通知App.vue登录失败
    const loginErrorEvent = new CustomEvent('login-error', {
      detail: { error }
    });
    window.dispatchEvent(loginErrorEvent);
  } finally {
    // 清除加载状态
    isLoading.value = false;
  }
}

// 切换密码可见性
function togglePasswordVisibility() {
  isPasswordVisible.value = !isPasswordVisible.value;
}

// 返回按钮处理
function goBack() {
  // 通知App.vue返回设置页面
  const event = new CustomEvent('navigate', { 
    detail: { page: 'settings' }
  });
  window.dispatchEvent(event);
}
</script>

<template>
  <!-- Main Content -->
  <main class="h-[600px] bg-gray-50">
    <div class="max-w-[400px] mx-auto h-full flex flex-col justify-center px-6">
      <!-- Login Form with Integrated Logo -->
      <div class="bg-white rounded-xl shadow-sm p-6">
        <div class="text-center mb-6">
          <div class="flex items-center justify-center space-x-2 mb-2">
            <i class="fa-solid fa-shield-halved text-pink-500 text-2xl"></i>
            <span class="text-xl font-bold text-gray-800">EasyVPN</span>
          </div>
          <p class="text-gray-500 text-sm">安全，快速的VPN服务</p>
        </div>
        
        <!-- 显示错误消息 -->
        <div v-if="errorMessage" class="p-2 bg-red-50 text-red-500 text-sm rounded mb-4">
          {{ errorMessage }}
        </div>
        
        <form class="space-y-4" @submit.prevent="handleLogin">
          <div>
            <label class="block text-sm text-gray-600 mb-1.5">账号</label>
            <div class="relative">
              <i class="fa-regular fa-user absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
              <input 
                v-model="username"
                type="text" 
                placeholder="请输入账号" 
                class="w-full pl-10 pr-3 py-2.5 border rounded-lg focus:outline-none focus:border-pink-500 text-sm"
              />
            </div>
          </div>
          <div>
            <label class="block text-sm text-gray-600 mb-1.5">密码</label>
            <div class="relative">
              <i class="fa-solid fa-lock absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
              <input 
                v-model="password"
                :type="isPasswordVisible ? 'text' : 'password'" 
                placeholder="请输入密码" 
                class="w-full pl-10 pr-3 py-2.5 border rounded-lg focus:outline-none focus:border-pink-500 text-sm"
              />
              <button 
                type="button"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                @click="togglePasswordVisibility"
              >
                <i :class="isPasswordVisible ? 'fa-regular fa-eye-slash' : 'fa-regular fa-eye'"></i>
              </button>
            </div>
          </div>
          <button 
            type="submit" 
            class="w-full bg-pink-500 text-white py-2.5 rounded-lg hover:bg-pink-600 font-medium flex items-center justify-center"
            :disabled="isLoading"
          >
            <i v-if="isLoading" class="fa-solid fa-spinner fa-spin mr-2"></i>
            {{ isLoading ? '登录中...' : '登录' }}
          </button>
        </form>
      </div>

      <!-- Back Link -->
      <div class="mt-6 text-center">
        <a href="#" class="text-sm text-gray-600 hover:text-gray-800" @click.prevent="goBack">
          <i class="fa-solid fa-arrow-left mr-1"></i>
          返回
        </a>
      </div>
    </div>
  </main>
</template> 