use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// 模拟键盘输入到当前焦点窗口
///
/// 使用剪贴板 + Ctrl+V 方式输入文本，支持中文和英文
#[tauri::command]
pub async fn type_text(app: AppHandle, text: String) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    // 保存当前剪贴板内容
    let saved_clipboard = save_clipboard(&app);

    // 将识别文本写入剪贴板
    app.clipboard()
        .write_text(&text)
        .map_err(|e| format!("写入剪贴板失败: {}", e))?;

    // 等待剪贴板写入完成
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 模拟 Ctrl+V
    simulate_ctrl_v();

    // 等待粘贴完成
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 恢复原剪贴板内容
    if let Some(saved) = saved_clipboard {
        let _ = app.clipboard().write_text(&saved);
    }

    Ok(())
}

/// 保存当前剪贴板文本内容
fn save_clipboard(app: &AppHandle) -> Option<String> {
    app.clipboard().read_text().ok()
}

/// 使用 Windows SendInput API 模拟 Ctrl+V
#[cfg(target_os = "windows")]
fn simulate_ctrl_v() {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_TYPE, VIRTUAL_KEY, VK_CONTROL, VK_V, KEYBD_EVENT_FLAGS,
    };

    let key_down = KEYBD_EVENT_FLAGS(0);
    let key_up = KEYBD_EVENT_FLAGS(2); // KEYEVENTF_KEYUP

    let mut inputs: [INPUT; 4] = unsafe { std::mem::zeroed() };

    // Ctrl 按下
    inputs[0].r#type = INPUT_TYPE(1);
    inputs[0].Anonymous.ki.wVk = VIRTUAL_KEY(VK_CONTROL.0 as u16);
    inputs[0].Anonymous.ki.dwFlags = key_down;

    // V 按下
    inputs[1].r#type = INPUT_TYPE(1);
    inputs[1].Anonymous.ki.wVk = VIRTUAL_KEY(VK_V.0 as u16);
    inputs[1].Anonymous.ki.dwFlags = key_down;

    // V 释放
    inputs[2].r#type = INPUT_TYPE(1);
    inputs[2].Anonymous.ki.wVk = VIRTUAL_KEY(VK_V.0 as u16);
    inputs[2].Anonymous.ki.dwFlags = key_up;

    // Ctrl 释放
    inputs[3].r#type = INPUT_TYPE(1);
    inputs[3].Anonymous.ki.wVk = VIRTUAL_KEY(VK_CONTROL.0 as u16);
    inputs[3].Anonymous.ki.dwFlags = key_up;

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(not(target_os = "windows"))]
fn simulate_ctrl_v() {
    // 非 Windows 平台暂不支持
}
