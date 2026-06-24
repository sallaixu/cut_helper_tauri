import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';


async function toggleWindowVisibility() {
    var appWindow = getCurrentWindow()
    const isMinimized = await appWindow.isMinimized();
    if (isMinimized) {
      await appWindow.unminimize();
      await appWindow.show();
      await appWindow.setFocus();
    } else {
      await appWindow.minimize();
    }
}

async function initSpeechHotkey() {
    const SPEECH_HOTKEY = 'Alt+Space';

    if (await isRegistered(SPEECH_HOTKEY)) {
        await unregister(SPEECH_HOTKEY);
    }

    await register(SPEECH_HOTKEY, async (event) => {
        if (event.state === 'Pressed') {
            try {
                // 尝试初始化模型（如果尚未加载）
                const status = await invoke('get_speech_status');
                if (!status.model_loaded) {
                    await invoke('init_speech_model', { lang: 'zh-en' });
                }
                await invoke('start_recording');
            } catch (e) {
                console.error('启动录音失败:', e);
            }
        } else if (event.state === 'Released') {
            try {
                const result = await invoke('stop_recording');
                if (result && result.trim()) {
                    await invoke('type_text', { text: result.trim() });
                }
            } catch (e) {
                console.error('停止录音失败:', e);
            }
        }
    });
}

export async function init_hotkey() {
    let focus_key = 'CommandOrControl+Space'
    if (await isRegistered(focus_key)) {
        await unregister(focus_key)
    }
    await register(focus_key, async (event) => {
        console.log("press")
        if (event.state === "Released") {
            console.log('Shortcut triggered');
            toggleWindowVisibility()
        }
    });

    // 注册语音输入快捷键
    await initSpeechHotkey();
}
