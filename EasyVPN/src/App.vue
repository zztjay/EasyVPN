<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import { ErrorMessages } from './constants/messages';
import { logToTerminal } from './utils/commonUtil';
import { useAccountStore } from './stores/account'
import { useVpnStore } from './stores/vpn'

// 使用 store 和 router
const accountStore = useAccountStore()
const vpnStore = useVpnStore()
const router = useRouter()

// 系统代理检查定时器
const proxyCheckInterval = ref(null as any);

// 组件挂载时初始化
onMounted(async () => {
  try {
    logToTerminal("App: 前端组件挂载时初始化");

    // 如果页面已经可见，直接获取账号数据
    if (document.visibilityState === 'visible') {
      logToTerminal("App: 窗口已可见，获取账号数据");
      await accountStore.fetchAccountData();
    } else {
      // 否则监听可见性变化
      const handleVisibilityChange = () => {
        if (document.visibilityState === 'visible') {
          logToTerminal("App: 监听：窗口变为可见，获取账号数据");
          document.removeEventListener('visibilitychange', handleVisibilityChange);
          accountStore.fetchAccountData();
        }
      };
      document.addEventListener('visibilitychange', handleVisibilityChange);
    }
    
    // 设置系统代理检查定时器（每10秒检查一次）
    proxyCheckInterval.value = setInterval(async () => {
      if (vpnStore.connectionStatus === "connected") {
        try {
          const result = await invoke('check_system_proxy');
          // 代理检查返回的是一个ProxyCheckCode枚举，需要检查它的值
          // 0表示正常(Ok)，其他值表示异常
          const proxyStatus = result;
          
          // 检查代理状态，非0表示异常
          if (proxyStatus !== "Ok") {
            logToTerminal(`App: 系统代理状态异常: ${proxyStatus}`);
            vpnStore.connectionStatus = "failed";
            vpnStore.errorMessage = ErrorMessages.PROXY_ERROR;
          }
        } catch (error) {
          logToTerminal(`App: 检查系统代理失败: ${error}`);
        }
      }

      // 更新账号数据
      await accountStore.updateAndFetchAccount();
      
    }, 10000);
    
    
    // 监听登录成功事件
    window.addEventListener('login-success', async () => {
      logToTerminal('App: 收到login-success事件');
      await accountStore.fetchAccountData();
    });
    
    // 添加监听退出登录成功事件
    window.addEventListener('logout-success', async () => {
      logToTerminal('App: 收到logout-success事件，刷新账号数据');
      await accountStore.fetchAccountData();
    });
    
  } catch (error) {
    vpnStore.connectionStatus = "failed";
    vpnStore.errorMessage = ErrorMessages.CLASH_START_FAILED;
    logToTerminal('App: 初始化失败, 错误详情:' + error);
  }
});

// 组件卸载时清理
onUnmounted(async () => {
  try {
    // 清除系统代理检查定时器
    if (proxyCheckInterval.value) {
      clearInterval(proxyCheckInterval.value);
    }
    
    // 清除事件监听
    window.removeEventListener('navigate', (event: any) => {});
    window.removeEventListener('login-success', () => {});
    window.removeEventListener('logout-success', () => {});
    
    // 停止Clash并关闭系统代理
    await invoke('stop_clash');
    logToTerminal('App: Clash已停止并关闭系统代理');
  } catch (error) {
    console.error('停止Clash失败:', error);
  }
});

// 监听用户状态变化，如果用户状态变为不允许连接的状态，则断开连接
watch(() => accountStore.userStatus, (newStatus: string, oldStatus: string) => {
  logToTerminal("App: 监听用户状态变化，新：" + newStatus + " ，老：" + oldStatus);

  // 如果用户状态变为允许连接，则连接VPN
  if(newStatus === "ON_SERVICE" || newStatus === "TRIAL"){
    if(vpnStore.connectionStatus === "failed"){
      vpnStore.connectionStatus = "disconnected";
    }
  }

  // 如果用户状态变为不允许连接，则断开连接
  if (vpnStore.connectionStatus === "connected" && 
      (newStatus !== "TRIAL" && newStatus !== "ON_SERVICE")) {
      vpnStore.toggleConnection();
  }
});
</script>

<template>
  <!-- 使用路由视图 -->
  <router-view class="w-full"></router-view>
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