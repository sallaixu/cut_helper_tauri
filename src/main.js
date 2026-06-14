import './assets/main.css'
import { createApp } from "vue";
import App from "./App.vue";

import router from './router/router'
import { start } from './cut_service'
import {init_hotkey} from './hotkey'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import todoNotifier from './todo_notifier'

window.addEventListener('error', (event) => {
    console.error('Uncaught error:', event.message);
});

async function main() {
    // 获取当前窗口标签
    const currentWindow = getCurrentWebviewWindow();
    const windowLabel = currentWindow.label;
    
    // 只在主窗口中初始化快捷键和剪贴板监控
    if (windowLabel === 'main') {
        await init_hotkey();
        start();
    }

    // 初始化待办通知调度器
    todoNotifier.init();
    
    const app = createApp(App);
    app.use(router);
    app.mount("#app");
}

main();