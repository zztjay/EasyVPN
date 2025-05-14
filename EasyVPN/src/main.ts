// @ts-ignore
import { createApp } from "vue";
import { createPinia } from "pinia";
import router from "./router";
import App from "./App.vue";

// 导入Tailwind CSS
import "./style.css";

// 导入Font Awesome
import "@fortawesome/fontawesome-free/css/all.min.css";

console.log("main.ts 执行");

// 创建应用实例
const app = createApp(App);

console.log("Vue应用创建完成");

// 注册Pinia和路由
app.use(createPinia());
app.use(router);

console.log("Pinia和Router注册完成");

// 错误处理
app.config.errorHandler = (err, vm, info) => {
  console.error("全局错误:", err);
  console.error("错误信息:", info);
};

// 挂载应用
app.mount("#app");

console.log("Vue应用挂载完成");
