use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
    Runtime,
    image::Image
};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    // 分割线
    let menu = Menu::with_items(app, &[&show_i, &hide_i, &settings_i, &quit_i])?;
    let tray_icon= Image::from_bytes(include_bytes!("../icons/icon.png"))
    .expect("Failed to load tray icon");
    let _ = TrayIconBuilder::with_id("tray")
        .icon(tray_icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            },
            "show" => {
                let window = app.get_webview_window("main").unwrap();
                let _ = window.show();
            },
            "hide" => {
                let window = app.get_webview_window("main").unwrap();
                let _ = window.hide();
            },
            "settings" => {
                // 打开设置窗口
                if let Some(window) = app.get_webview_window("settings") {
                    // 如果设置窗口已存在，显示并聚焦
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    // 创建新的设置窗口
                    use tauri::WebviewWindowBuilder;
                    let _ = WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::App("/#/settings".into()))
                        .title("设置")
                        .inner_size(600.0, 450.0)
                        .resizable(true)
                        .center()
                        .build();
                }
            },
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app);

    Ok(())
}

/// 切换托盘图标以表示录音状态
pub fn set_recording_icon<R: Runtime>(app: &tauri::AppHandle<R>, is_recording: bool) {
    if let Some(_tray) = app.tray_by_id("tray") {
        if is_recording {
            // 录音中：使用系统通知
            use tauri_plugin_notification::NotificationExt;
            let _ = app.notification()
                .builder()
                .title("剪切板助手")
                .body("正在录音，松开快捷键结束")
                .show();
        }
    }
}

