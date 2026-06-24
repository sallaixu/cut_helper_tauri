use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use lazy_static::lazy_static;

use sherpa_onnx::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};

/// 识别器状态（单独 Mutex，避免与 stream 的借用冲突）
struct RecognizerState {
    recognizer: Option<OnlineRecognizer>,
    model_loaded: bool,
    current_lang: String,
}

impl Default for RecognizerState {
    fn default() -> Self {
        Self {
            recognizer: None,
            model_loaded: false,
            current_lang: String::new(),
        }
    }
}

/// 录音状态（单独 Mutex）
struct RecordingState {
    stream: Option<OnlineStream>,
    is_recording: bool,
    stop_flag: bool,
    final_text: String,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self {
            stream: None,
            is_recording: false,
            stop_flag: false,
            final_text: String::new(),
        }
    }
}

lazy_static! {
    static ref RECOGNIZER_STATE: Mutex<RecognizerState> = Mutex::new(RecognizerState::default());
    static ref RECORDING_STATE: Mutex<RecordingState> = Mutex::new(RecordingState::default());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechStatus {
    pub model_loaded: bool,
    pub is_recording: bool,
    pub current_lang: String,
}

/// 初始化（加载）指定语言的语音识别模型
#[tauri::command]
pub async fn init_speech_model(app: AppHandle, lang: String) -> Result<(), String> {
    let model_dir = super::model_manager::get_model_dir(&app, &lang)?;

    if !model_dir.exists() {
        return Err(format!(
            "模型未下载，请先在设置中下载 {} 模型",
            lang
        ));
    }

    // 根据语言配置识别器
    let mut config = OnlineRecognizerConfig::default();
    config.model_config.tokens = Some(model_dir.join("tokens.txt").to_string_lossy().to_string());

    if lang == "zh-en" {
        // 双语 Zipformer transducer 模型
        config.model_config.transducer.encoder = Some(
            model_dir
                .join("encoder-epoch-99-avg-1.int8.onnx")
                .to_string_lossy()
                .to_string(),
        );
        config.model_config.transducer.decoder = Some(
            model_dir
                .join("decoder-epoch-99-avg-1.onnx")
                .to_string_lossy()
                .to_string(),
        );
        config.model_config.transducer.joiner = Some(
            model_dir
                .join("joiner-epoch-99-avg-1.int8.onnx")
                .to_string_lossy()
                .to_string(),
        );
    }

    config.enable_endpoint = true;
    config.decoding_method = Some("greedy_search".to_string());

    let recognizer = match OnlineRecognizer::create(&config) {
        Some(r) => r,
        None => return Err("创建识别器失败".to_string()),
    };

    let mut state = RECOGNIZER_STATE
        .lock()
        .map_err(|e| format!("获取状态锁失败: {}", e))?;
    state.recognizer = Some(recognizer);
    state.current_lang = lang;
    state.model_loaded = true;

    Ok(())
}

/// 开始录音和流式识别
#[tauri::command]
pub async fn start_recording(app: AppHandle) -> Result<(), String> {
    let recognizer_state = RECOGNIZER_STATE
        .lock()
        .map_err(|e| format!("获取识别器状态锁失败: {}", e))?;

    if !recognizer_state.model_loaded || recognizer_state.recognizer.is_none() {
        let lang = if recognizer_state.current_lang.is_empty() {
            "zh-en".to_string()
        } else {
            recognizer_state.current_lang.clone()
        };
        return Err(format!(
            "语音模型未加载，请先初始化 {} 模型",
            lang
        ));
    }

    // 创建新的识别流
    let stream = recognizer_state.recognizer.as_ref().unwrap().create_stream();

    // 更新录音状态
    {
        let mut rec_state = RECORDING_STATE
            .lock()
            .map_err(|e| format!("获取录音状态锁失败: {}", e))?;
        if rec_state.is_recording {
            return Ok(());
        }

        rec_state.stream = Some(stream);
        rec_state.is_recording = true;
        rec_state.stop_flag = false;
        rec_state.final_text = String::new();
    }

    // 推送录音状态
    let _ = app.emit("recording-state", serde_json::json!({ "is_recording": true }));

    // 在后台线程中启动音频采集和识别
    let app_handle = app.clone();
    std::thread::spawn(move || {
        recording_loop(app_handle);
    });

    Ok(())
}

/// 录音和识别循环（在后台线程中运行）
fn recording_loop(app: AppHandle) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    // 获取默认音频输入设备
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => d,
        None => {
            let _ = app.emit(
                "recording-state",
                serde_json::json!({ "is_recording": false }),
            );
            eprintln!("未找到音频输入设备");
            return;
        }
    };

    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("获取音频配置失败: {}", e);
            let _ = app.emit(
                "recording-state",
                serde_json::json!({ "is_recording": false }),
            );
            return;
        }
    };

    let sample_rate = config.sample_rate().0;

    // 创建音频采集的通道
    let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();

    // 构建音频输入流
    let stream_result = match config.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let _ = tx.send(data.to_vec());
                },
                |err| eprintln!("音频采集错误: {}", err),
                None,
            ),
        cpal::SampleFormat::I16 => {
            let tx_i16 = tx.clone();
            device
                .build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let f32_data: Vec<f32> =
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        let _ = tx_i16.send(f32_data);
                    },
                    |err| eprintln!("音频采集错误: {}", err),
                    None,
                )
        }
        fmt => {
            eprintln!("不支持的音频格式: {:?}", fmt);
            let _ = app.emit(
                "recording-state",
                serde_json::json!({ "is_recording": false }),
            );
            return;
        }
    };

    let stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            eprintln!("创建音频流失败: {}", e);
            let _ = app.emit(
                "recording-state",
                serde_json::json!({ "is_recording": false }),
            );
            return;
        }
    };

    // 启动音频采集
    if let Err(e) = stream.play() {
        eprintln!("启动音频采集失败: {}", e);
        let _ = app.emit(
            "recording-state",
            serde_json::json!({ "is_recording": false }),
        );
        return;
    }

    // 重采样目标 16kHz
    let target_sample_rate: u32 = 16000;

    // 识别循环
    loop {
        // 检查停止标志
        {
            let rec_state = match RECORDING_STATE.lock() {
                Ok(s) => s,
                Err(_) => break,
            };
            if rec_state.stop_flag {
                break;
            }
        }

        // 从音频通道读取数据（超时 100ms）
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(samples) => {
                // 重采样
                let resampled = if sample_rate != target_sample_rate {
                    simple_resample(&samples, sample_rate, target_sample_rate)
                } else {
                    samples
                };

                // 送入识别器 — 分别获取 recognizer 和 stream 的引用
                let result_text = {
                    // 先获取 recognizer 的锁
                    let recognizer_guard = match RECOGNIZER_STATE.lock() {
                        Ok(s) => s,
                        Err(_) => break,
                    };
                    let recognizer = match &recognizer_guard.recognizer {
                        Some(r) => r,
                        None => break,
                    };

                    // 再获取 stream 的锁
                    let mut stream_guard = match RECORDING_STATE.lock() {
                        Ok(s) => s,
                        Err(_) => break,
                    };
                    let stream = match &mut stream_guard.stream {
                        Some(s) => s,
                        None => break,
                    };

                    stream.accept_waveform(target_sample_rate as i32, &resampled);

                    while recognizer.is_ready(stream) {
                        recognizer.decode(stream);
                    }

                    let text = recognizer.get_result(stream)
                        .map(|r| r.text.clone())
                        .unwrap_or_default();

                    if recognizer.is_endpoint(stream) {
                        recognizer.reset(stream);
                    }

                    text
                };

                // 更新状态和推送事件
                if !result_text.is_empty() {
                    let _ = app.emit(
                        "speech-partial-result",
                        serde_json::json!({ "text": &result_text }),
                    );
                    let mut rec_state = match RECORDING_STATE.lock() {
                        Ok(s) => s,
                        Err(_) => break,
                    };
                    rec_state.final_text = result_text;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // 停止音频采集（drop stream）
    drop(stream);

    // 推送录音结束状态
    let _ = app.emit(
        "recording-state",
        serde_json::json!({ "is_recording": false }),
    );
}

/// 简单的线性重采样
fn simple_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = to_rate as f64 / from_rate as f64;
    let new_len = (samples.len() as f64 * ratio) as usize;
    let mut result = Vec::with_capacity(new_len);
    for i in 0..new_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx as usize;
        if idx + 1 < samples.len() {
            let frac = src_idx - idx as f64;
            result.push(samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32);
        } else if idx < samples.len() {
            result.push(samples[idx]);
        }
    }
    result
}

/// 停止录音，返回最终识别文本
#[tauri::command]
pub async fn stop_recording() -> Result<String, String> {
    let mut rec_state = RECORDING_STATE
        .lock()
        .map_err(|e| format!("获取状态锁失败: {}", e))?;

    if !rec_state.is_recording {
        return Ok(String::new());
    }

    // 设置停止标志
    rec_state.stop_flag = true;
    rec_state.is_recording = false;

    // 获取最终文本
    let final_text = rec_state.final_text.clone();
    rec_state.final_text = String::new();

    // 清理流
    rec_state.stream = None;

    Ok(final_text)
}

/// 获取语音识别状态
#[tauri::command]
pub fn get_speech_status() -> Result<SpeechStatus, String> {
    let recognizer_state = RECOGNIZER_STATE
        .lock()
        .map_err(|e| format!("获取识别器状态锁失败: {}", e))?;
    let recording_state = RECORDING_STATE
        .lock()
        .map_err(|e| format!("获取录音状态锁失败: {}", e))?;

    Ok(SpeechStatus {
        model_loaded: recognizer_state.model_loaded,
        is_recording: recording_state.is_recording,
        current_lang: recognizer_state.current_lang.clone(),
    })
}
