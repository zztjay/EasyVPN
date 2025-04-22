<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'

// 接收来自父组件的到期时间
const props = defineProps({
  expiryDate: { 
    type: [String, Date], 
    default: () => {
      // 默认7天后到期
      const date = new Date();
      date.setDate(date.getDate() + 7);
      return date.toISOString();
    }
  }
});

// 剩余总秒数
const remainingSeconds = ref(0);

// 计算剩余秒数
function calculateRemainingSeconds() {
  const now = new Date();
  const expiry = new Date(props.expiryDate);
  
  // 计算时间差（毫秒）
  const diffMs = expiry.getTime() - now.getTime();
  
  // 如果到期时间已过，则返回0
  if (diffMs <= 0) {
    return 0;
  }
  
  // 转换为秒
  return Math.floor(diffMs / 1000);
}

// 通过计算属性获取天、时、分、秒
const trialDays = computed(() => Math.floor(remainingSeconds.value / 86400));
const trialHours = computed(() => Math.floor((remainingSeconds.value % 86400) / 3600));
const trialMinutes = computed(() => Math.floor((remainingSeconds.value % 3600) / 60));
const trialSeconds = computed(() => remainingSeconds.value % 60);

// 定时器
let timerInterval: number | undefined = undefined;

// 更新试用时间
function updateTrialTime() {
  remainingSeconds.value = calculateRemainingSeconds();
  
  // 试用期结束
  if (remainingSeconds.value <= 0 && timerInterval !== undefined) {
    clearInterval(timerInterval);
    timerInterval = undefined;
  }
}

onMounted(() => {
  // 初始计算剩余时间
  remainingSeconds.value = calculateRemainingSeconds();
  
  // 开始计时器
  timerInterval = setInterval(updateTrialTime, 1000);
});

onUnmounted(() => {
  // 清除计时器
  if (timerInterval !== undefined) {
    clearInterval(timerInterval);
    timerInterval = undefined;
  }
});
</script>

<template>
  <div id="trial-timer" class="bg-pink-50 rounded-lg p-4">
    <div class="flex items-center justify-center space-x-2 mb-2">
      <i class="fa-regular fa-clock text-pink-500"></i>
      <h3 class="text-sm font-medium text-pink-700">剩余试用期</h3>
    </div>
    <div class="text-2xl font-bold text-pink-600">
      {{ trialDays }}天 {{ trialHours.toString().padStart(2, '0') }}:{{ trialMinutes.toString().padStart(2, '0') }}:{{ trialSeconds.toString().padStart(2, '0') }}
    </div>
  </div>
</template> 