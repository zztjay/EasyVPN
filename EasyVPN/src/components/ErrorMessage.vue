<script setup lang="ts">
defineProps({
  message: {
    type: String,
    default: '请检查网络连接后重试'
  }
});
</script>

<template>
  <div id="error-message" class="mb-4 mt-0 bg-red-50 border border-red-200 rounded-lg p-3">
    <div class="flex items-start space-x-3">
      <i class="fa-solid fa-circle-exclamation text-red-500 mt-0.5"></i>
      <div class="flex-1">
        <p class="text-sm text-red-600">{{ message }}</p>
      </div>
    </div>
  </div>
</template> 