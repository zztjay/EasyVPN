/// <reference types="vite/client" />

// 全局组件类型声明
declare module '*.vue' {
  import type { ComponentOptions } from 'vue'
  const component: ComponentOptions
  export default component
}

// 添加Vue API类型
declare module 'vue' {
  export interface GlobalComponents {
    // 可以在此添加全局组件
  }
  
  // 确保Vue API可用
  export const ref: any
  export const computed: any
  export const reactive: any
  export const onMounted: any
  export const onUnmounted: any
  export const watch: any
  export const defineProps: any
  export const defineEmits: any
  export const defineExpose: any
  export const watchEffect: any
} 