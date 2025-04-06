<script setup lang="ts">
// 接收剩余天数
defineProps({
  serviceDays: { type: Number, default: 27 }
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
        <span class="text-2xl font-bold text-pink-600">{{ serviceDays }}</span>
        <span class="text-sm text-gray-600">天剩余</span>
      </div>
    </div>
  </div>
</template> 