<script setup lang="ts">
import { ref } from 'vue'

// 接收价格参数
const props = defineProps({
  price: { type: Number, default: 69.99 }
});

// 购买服务事件
const emit = defineEmits(['purchase']);

function handlePurchase() {
  emit('purchase');
}
</script>

<template>
  <div id="purchase-option" class="bg-pink-50 rounded-lg p-4">
    <div class="flex items-center justify-center space-x-2 mb-3">
      <i class="fa-regular fa-gem text-pink-500"></i>
      <h3 class="text-sm font-medium text-pink-700">服务已到期</h3>
    </div>
    <button 
      class="w-full bg-white hover:bg-pink-50 text-pink-600 border-2 border-pink-200 rounded-lg py-3 transition-colors duration-200 flex items-center justify-center space-x-2"
      @click="handlePurchase"
    >
      <i class="fa-regular fa-credit-card"></i>
      <span class="font-medium">立即购买 - ¥{{ price }}/月</span>
    </button>
  </div>
</template> 