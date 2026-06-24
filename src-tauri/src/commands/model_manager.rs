use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;

/// 模型配置
struct ModelConfig {
    lang: &'static str,
    display_name: &'static str,
    huggingface_repo: &'static str,
    /// 模型目录中必须存在的文件列表，用于验证下载完整性
    required_files: &'static [&'static str],
}

/// 所有可用模型配置
const MODELS: &[ModelConfig] = &[
    ModelConfig {
        lang: "zh-en",
        display_name: "中英双语",
        huggingface_repo: "csukuangfj/sherpa-onnx-streaming-zipformer-small-bilingual-zh-en-2023-02-16",
        required_files: &[
            "encoder-epoch-99-avg-1.int8.onnx",
            "decoder-epoch-99-avg-1.onnx",
            "joiner-epoch-99-avg-1.int8.onnx",
            "tokens.txt",
        ],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub lang: String,
    pub display_name: String,
    pub size_mb: f64,
    pub status: String, // "not_downloaded" | "downloading" | "ready"
}

/// 获取模型存储根目录
fn get_models_root(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    Ok(app_data_dir.join("speech_models"))
}

/// 获取指定语言的模型目录
pub fn get_model_dir(app: &AppHandle, lang: &str) -> Result<PathBuf, String> {
    Ok(get_models_root(app)?.join(lang))
}

/// 查找模型配置
fn find_model_config(lang: &str) -> Result<&'static ModelConfig, String> {
    MODELS
        .iter()
        .find(|m| m.lang == lang)
        .ok_or_else(|| format!("未知语言: {}", lang))
}

/// 检查模型是否已完整下载
fn is_model_ready(model_dir: &PathBuf, required_files: &[&str]) -> bool {
    required_files
        .iter()
        .all(|f| model_dir.join(f).exists())
}

/// 计算模型目录大小（MB）
fn calculate_model_size(model_dir: &PathBuf) -> f64 {
    let mut total_size: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(model_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }
    }
    total_size as f64 / (1024.0 * 1024.0)
}

/// 列出所有可用模型及其状态
#[tauri::command]
pub async fn list_speech_models(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let models_root = get_models_root(&app)?;
    let mut result = Vec::new();

    for config in MODELS {
        let model_dir = models_root.join(config.lang);
        let (status, size_mb) = if is_model_ready(&model_dir, config.required_files) {
            ("ready".to_string(), calculate_model_size(&model_dir))
        } else if model_dir.exists() {
            ("downloading".to_string(), calculate_model_size(&model_dir))
        } else {
            ("not_downloaded".to_string(), 0.0)
        };

        result.push(ModelInfo {
            lang: config.lang.to_string(),
            display_name: config.display_name.to_string(),
            size_mb,
            status,
        });
    }

    Ok(result)
}

/// 下载指定语言的模型
#[tauri::command]
pub async fn download_speech_model(app: AppHandle, lang: String) -> Result<(), String> {
    let config = find_model_config(&lang)?;
    let models_root = get_models_root(&app)?;
    let model_dir = models_root.join(&lang);

    // 创建模型目录
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| format!("创建模型目录失败: {}", e))?;

    // 逐个下载必需文件
    let client = reqwest::Client::new();
    let base_url = format!(
        "https://huggingface.co/{}/resolve/main",
        config.huggingface_repo
    );

    let total_files = config.required_files.len();
    for (i, file_name) in config.required_files.iter().enumerate() {
        let url = format!("{}/{}", base_url, file_name);
        let file_path = model_dir.join(file_name);

        // 如果文件已存在且大小 > 0，跳过
        if let Ok(metadata) = std::fs::metadata(&file_path) {
            if metadata.len() > 0 {
                continue;
            }
        }

        // 下载文件
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("下载 {} 失败: {}", file_name, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "下载 {} 返回错误状态: {}",
                file_name,
                response.status()
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|e| format!("创建文件 {} 失败: {}", file_name, e))?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("读取数据失败: {}", e))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("写入文件失败: {}", e))?;
            downloaded += chunk.len() as u64;

            // 推送下载进度
            let progress = if total_size > 0 {
                downloaded as f64 / total_size as f64
            } else {
                0.0
            };
            let overall_progress = (i as f64 + progress) / total_files as f64;

            let _ = app.emit(
                "model-download-progress",
                serde_json::json!({
                    "lang": &lang,
                    "progress": overall_progress,
                    "current_file": file_name,
                }),
            );
        }
    }

    Ok(())
}

/// 删除指定语言的模型
#[tauri::command]
pub async fn delete_speech_model(app: AppHandle, lang: String) -> Result<(), String> {
    let model_dir = get_model_dir(&app, &lang)?;

    if !model_dir.exists() {
        return Err("模型目录不存在".to_string());
    }

    std::fs::remove_dir_all(&model_dir)
        .map_err(|e| format!("删除模型失败: {}", e))?;

    Ok(())
}
