<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';

// 接收到期时间
const props = defineProps({
  expiryDate: { 
    type: [String, Date], 
    default: () => {
      // 默认27天后到期
      const date = new Date();
      date.setDate(date.getDate() + 27);
      return date.toISOString();
    }
  }
});

// 剩余天数
const remainingDays = ref(0);

// 计算剩余天数
function calculateRemainingDays() {
  const now = new Date();
  const expiry = new Date(props.expiryDate);
  
  // 重置时间部分以便准确计算天数差异
  now.setHours(0, 0, 0, 0);
  expiry.setHours(0, 0, 0, 0);
  
  // 计算时间差（毫秒）
  const diffMs = expiry.getTime() - now.getTime();
  
  // 如果到期时间已过，则返回0
  if (diffMs <= 0) {
    return 0;
  }
  
  // 转换为天数（1天 = 24 * 60 * 60 * 1000毫秒）
  return Math.ceil(diffMs / (24 * 60 * 60 * 1000));
}

// 定时器
let updateInterval: number | undefined = undefined;

// 更新剩余天数
function updateRemainingDays() {
  remainingDays.value = calculateRemainingDays();
}

onMounted(() => {
  // 初始计算剩余天数
  remainingDays.value = calculateRemainingDays();
  
  // 设置定时器每天凌晨更新
  const updateAtMidnight = () => {
    const now = new Date();
    const night = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate() + 1, // 下一天
      0, 0, 0 // 凌晨 00:00:00
    );
    const msToMidnight = night.getTime() - now.getTime();
    
    // 设置在凌晨时更新，之后每24小时更新一次
    setTimeout(() => {
      updateRemainingDays();
      updateInterval = setInterval(updateRemainingDays, 24 * 60 * 60 * 1000);
    }, msToMidnight);
  };
  
  updateAtMidnight();
});

onUnmounted(() => {
  // 清除定时器
  if (updateInterval !== undefined) {
    clearInterval(updateInterval);
    updateInterval = undefined;
  }
});
</script>

<template>
  <!-- Service Time Remaining -->
  <div id="service-time" class="bg-pink-50 rounded-lg p-4">
    <div class="flex items-center justify-center space-x-2 mb-3">
      <i class="fa-regular fa-clock text-pink-500"></i>
      <h3 class="text-sm font-medium text-pink-700">剩余服务时间</h3>
    </div>
    <div class="bg-white border-2 border-pink-200 rounded-lg py-3 px-4">
      <div class="flex items-center justify-center space-x-2">
        <span class="text-2xl font-bold text-pink-600">{{ remainingDays }}</span>
        <span class="text-sm text-gray-600">天剩余</span>
      </div>
    </div>
  </div>
</template> 