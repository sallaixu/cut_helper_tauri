pub mod commands;
pub mod utils;
pub mod config;
use tauri_plugin_sql::{Migration, MigrationKind};
use tauri::Listener;
mod tray;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        // 版本2 - 原始的初始表创建（必须保留，不能修改，否则数据库会报错）
        Migration {
            version: 2,
            description: "create_initial_tables",
            sql: r#"
            CREATE TABLE  IF NOT EXISTS "CutItems" (
                "id" UUID NOT NULL,
                "content" TEXT NOT NULL,
                "createTime" DATETIME NOT NULL,
                PRIMARY KEY ("id")
              );

            CREATE TABLE IF NOT EXISTS "Groups" (
            "id" UUID NOT NULL,
            "name" VARCHAR(255) NOT NULL,
            "createTime" DATETIME NOT NULL,
            PRIMARY KEY ("id")
            );

            CREATE TABLE IF NOT EXISTS "GroupItems" (
            "id" UUID NOT NULL,
            "groupId" VARCHAR(255) NOT NULL,
            "content" TEXT NOT NULL,
            "title" VARCHAR(255),
            "createTime" DATETIME NOT NULL,
            "updateTime" DATETIME,
            PRIMARY KEY ("id")
            );

            "#,
            kind: MigrationKind::Up,
        },
        // 版本4 - 添加图片表（跳过版本3避免之前的冲突）
        Migration {
            version: 4,
            description: "add_image_items_table",
            sql: r#"
            CREATE TABLE IF NOT EXISTS "ImageItems" (
            "id" UUID NOT NULL,
            "content" TEXT NOT NULL,
            "width" INTEGER,
            "height" INTEGER,
            "size" INTEGER,
            "createTime" DATETIME NOT NULL,
            PRIMARY KEY ("id")
            );
            "#,
            kind: MigrationKind::Up,
        },
        // 版本5 - 添加待办事项表
        Migration {
            version: 5,
            description: "add_todo_items_table",
            sql: r#"
            CREATE TABLE IF NOT EXISTS "TodoItems" (
                "id" UUID NOT NULL,
                "title" VARCHAR(255) NOT NULL,
                "note" TEXT,
                "status" VARCHAR(20) NOT NULL DEFAULT 'pending',
                "startTime" DATETIME,
                "endTime" DATETIME,
                "duration" INTEGER,
                "notifyStart" BOOLEAN NOT NULL DEFAULT 1,
                "notifyEnd" BOOLEAN NOT NULL DEFAULT 1,
                "notifyAdvance" INTEGER NOT NULL DEFAULT 5,
                "createTime" DATETIME NOT NULL,
                "updateTime" DATETIME,
                PRIMARY KEY ("id")
            );
            "#,
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:cut.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(all(desktop))]
            {
            let handle = app.handle();
            tray::create_tray(handle)?;
            
            // 根据配置设置自启动
            let config_result = config::AppConfig::load(&handle);
            if let Ok(cfg) = config_result {
                use tauri_plugin_autostart::ManagerExt;
                let auto_launch = handle.autolaunch();
                
                if cfg.auto_start {
                    let _ = auto_launch.enable();
                } else {
                    let _ = auto_launch.disable();
                }
            }
            }

            // 监听录音状态事件，更新托盘通知
            #[cfg(target_os = "windows")]
            {
                let tray_handle = app.handle().clone();
                app.listen("recording-state", move |event| {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                        if let Some(is_recording) = data.get("is_recording").and_then(|v| v.as_bool()) {
                            tray::set_recording_icon(&tray_handle, is_recording);
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::cut_admin::test_fun,
            commands::cut_admin::get_db_path,
            commands::image_processor::process_clipboard_image,
            commands::image_processor::calculate_image_hash,
            commands::image_processor::monitor_and_process_clipboard_image,
            #[cfg(target_os = "windows")]
            commands::keyboard_sim::type_text,
            #[cfg(target_os = "windows")]
            commands::model_manager::list_speech_models,
            #[cfg(target_os = "windows")]
            commands::model_manager::download_speech_model,
            #[cfg(target_os = "windows")]
            commands::model_manager::delete_speech_model,
            #[cfg(target_os = "windows")]
            commands::speech::init_speech_model,
            #[cfg(target_os = "windows")]
            commands::speech::start_recording,
            #[cfg(target_os = "windows")]
            commands::speech::stop_recording,
            #[cfg(target_os = "windows")]
            commands::speech::get_speech_status,
            config::get_config,
            config::save_config,
            config::set_auto_start,
            config::is_auto_start_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
