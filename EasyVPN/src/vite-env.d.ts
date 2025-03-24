/// <reference types="vite/client" />
/// <reference types="vue" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

// 增强模板内绑定属性的类型检查
declare module "vue" {
  interface ComponentCustomProperties {
    $props: Record<string, any>;
    $attrs: Record<string, any>;
    $emit: (event: string, ...args: any[]) => void;
    $slots: Record<string, any>;
    $refs: Record<string, any>;
    $root: ComponentPublicInstance;
    
    // 添加响应式引用类型
    [key: string]: any;
  }
}
