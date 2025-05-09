use crate::utils::custom_result::CustomResult;
use crate::utils::sub_marker::generate_srt;
use crate::utils::tts::TTS;
use base64::{engine::general_purpose, Engine as _};
use futures_util::{sink::SinkExt, StreamExt};
use reqwest::get;
use serde::Deserialize;
use serde_json::Value;
use serde_json::{from_str, json};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TTSData {
    voice: String,
    text: String,
    pitch: i32,
    rate: i32,
    volume: i32,
    gender_sub_marker: bool,
    sub_marker_type: String,
    merge_by_number_number: i32,
    root_path: String,
    open_folders: bool,
}

#[tauri::command]
pub async fn get_voices_list() -> Result<CustomResult, CustomResult> {
    let url = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

    let response = get(url)
        .await
        .map_err(|e| CustomResult::error(Some(e.to_string()), None))?;

    if response.status().is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| CustomResult::error(Some(e.to_string()), None))?;
        let json: serde_json::Value =
            from_str(body.as_str()).map_err(|e| CustomResult::error(Some(e.to_string()), None))?;
        return Ok(CustomResult::success(None, Some(json)));
    }

    return Err(CustomResult::error(
        Some(response.status().to_string()),
        None,
    ));
}

#[tauri::command]
pub async fn start_tts(data: TTSData) -> Result<CustomResult, CustomResult> {
    let voice = data.voice;
    let text = data.text;
    let pitch = data.pitch;
    let rate = data.rate;
    let volume = data.volume;
    let sub_marker_switch = data.gender_sub_marker;
    let sub_marker_option = data.sub_marker_type;
    let merge_number = data.merge_by_number_number;
    let mut root_path = data.root_path;
    let open_folders = data.open_folders;

    // 检查参数
    if voice.is_empty() || text.is_empty() || sub_marker_option.is_empty() {
        return Err(CustomResult::error(Some("参数错误".to_string()), None));
    }

    // 格式化数据
    let pitch_str = if pitch >= 0 {
        format!("+{}", pitch)
    } else {
        format!("{}", pitch)
    };

    let rate_str = if rate >= 0 {
        format!("+{}", rate)
    } else {
        format!("{}", rate)
    };

    let volumn_str = if volume >= 0 {
        format!("+{}", volume)
    } else {
        format!("{}", volume)
    };

    let tts_client = TTS {};
    let send_request_id = Uuid::new_v4().to_string().replace('-', "");
    let sec_ms_gec_value = tts_client.generate_sec_ms_gec()?.data["hax"].clone();
    let sec_ms_gec = sec_ms_gec_value
        .as_str()
        .ok_or_else(|| CustomResult::error(Some("生成令牌字符串失败".to_string()), None))?;
    let audio_output_format = "audio-24khz-48kbitrate-mono-mp3";
    let binary_delim = "Path:audio\r\n";

    let url_str = format!(
        "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?\
         TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&Sec-MS-GEC={}&Sec-MS-GEC-Version=1-130.0.2849.68&ConnectionId={}",
        sec_ms_gec, send_request_id
    );

    let mut request = url_str
        .into_client_request()
        .map_err(|e| CustomResult::error(Some(format!("编码URL失败：{}", e.to_string())), None))?;

    // 添加请求头
    request
        .headers_mut()
        .insert("Pragma", "no-cache".parse().unwrap());
    request
        .headers_mut()
        .insert("Cache-Control", "no-cache".parse().unwrap());
    request.headers_mut().insert(
        "Origin",
        "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"
            .parse()
            .unwrap(),
    );
    request.headers_mut().insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0".parse().unwrap());
    request
        .headers_mut()
        .insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    request
        .headers_mut()
        .insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());

    let (mut socket, _) = connect_async(request)
        .await
        .map_err(|e| CustomResult::error(Some(format!("连接wss失败：{}", e.to_string())), None))?;

    // 发送音频格式设定
    let audio_config = tts_client.convert_to_audio_format_websocket_string(audio_output_format);
    socket
        .send(Message::Text(audio_config.into()))
        .await
        .map_err(|e| CustomResult::error(Some(format!("发送音频格式失败：{}", e)), None))?;

    // 发送 SSML 文本
    let ssml = tts_client.convert_to_ssml_websocket_string(
        &send_request_id,
        &voice,
        &text,
        &pitch_str,
        &rate_str,
        &volumn_str,
    );
    socket
        .send(Message::Text(ssml.into()))
        .await
        .map_err(|e| CustomResult::error(Some(format!("发送SSML文本失败：{}", e)), None))?;

    // 接收数据
    let mut audio_data: Vec<u8> = Vec::new();
    let mut messages: Vec<Value> = vec![];

    while let Some(msg) = socket.next().await {
        let msg =
            msg.map_err(|e| CustomResult::error(Some(format!("读取WS输出失败：{}", e)), None))?;

        match msg {
            Message::Text(txt) => {
                if txt.contains("Path:turn.end") {
                    break;
                } else if txt.contains("Path:audio.metadata") {
                    // 找到第一个空行之后的第一个非空行，这应该是 JSON 数据的开始
                    if let Some(start_index) = txt.find("\r\n\r\n") {
                        let json_start = start_index + 2; // 跳过两个换行符
                        let json_part = &txt[json_start..];
                        // 尝试解析 JSON
                        if let Ok(json) = serde_json::from_str(json_part) {
                            messages.push(json);
                        } else {
                            eprintln!("JSON 解析失败: {}", json_part);
                        }
                    }
                }
            }
            Message::Binary(bin) => {
                if let Some(index) = bin
                    .windows(binary_delim.len())
                    .position(|w| w == binary_delim.as_bytes())
                {
                    audio_data.extend_from_slice(&bin[index + binary_delim.len()..]);
                }
            }
            _ => {}
        }
    }

    if root_path.is_empty() {
        root_path = ".".to_string();
    }

    let path_str = format!("{}/{}", root_path, send_request_id);
    let folder_path = Path::new(&path_str);
    if !folder_path.exists() {
        fs::create_dir_all(folder_path)
            .map_err(|e| CustomResult::error(Some(format!("创建文件夹失败：{}", e)), None))?;
    }

    // 保存音频数据
    let output_path = format!("{}/output_{}.mp3", path_str, send_request_id);
    let mut file = File::create(&output_path)
        .map_err(|e| CustomResult::error(Some(format!("创建音频文件失败：{}", e)), None))?;
    file.write_all(&audio_data)
        .map_err(|e| CustomResult::error(Some(format!("写入音频文件失败：{}", e)), None))?;

    // 写入JSON数据（仅用于测试）
    // let json_path = format!("{}/output_{}.json", path_str, send_request_id);
    // let mut json_file = File::create(&json_path)
    //     .map_err(|e| CustomResult::error(Some(format!("创建JSON文件失败：{}", e)), None))?;
    // let json_str = serde_json::to_string(&messages)
    //     .map_err(|e| CustomResult::error(Some(format!("序列化JSON失败：{}", e)), None))?;
    // json_file
    //     .write_all(json_str.as_bytes())
    //     .map_err(|e| CustomResult::error(Some(format!("写入JSON文件失败：{}", e)), None))?;

    // 写入文本数据（仅用于测试）
    // let text_path = format!("{}/output_{}.txt", path_str, send_request_id);
    // let mut text_file = File::create(&text_path)
    //     .map_err(|e| CustomResult::error(Some(format!("创建文本文件失败：{}", e)), None))?;
    // let text = text.replace("\n", "\r\n");
    // text_file
    //     .write_all(text.as_bytes())
    //     .map_err(|e| CustomResult::error(Some(format!("写入文本文件失败：{}", e)), None))?;

    if sub_marker_switch {
        let _ = generate_srt(
            &messages,
            &format!("{}/output_{}.srt", path_str, send_request_id),
            &sub_marker_option,
            merge_number,
            Some(&text),
        );
    }

    if open_folders {
        // 打开文件夹
        let final_path = if cfg!(target_os = "windows") {
            path_str.replace("/", "\\")
        } else {
            path_str
        };
        Command::new("explorer")
            .arg(Path::new(final_path.as_str()))
            .status()
            .map_err(|e| {
                CustomResult::error(
                    Some(format!(
                        "打开文件夹失败：{}<br>请手动前往文件夹查看：{}",
                        e, final_path
                    )),
                    None,
                )
            })?;
    }

    // 编码成base64
    let base64_audio = encode_audio_to_base64(&output_path)?;

    Ok(CustomResult::success(
        None,
        Some(json!({"audio": base64_audio})),
    ))
}

#[tauri::command]
pub async fn get_exe_path() -> Result<CustomResult, CustomResult> {
    let exe_path = std::env::current_exe()
        .map_err(|e| CustomResult::error(Some(format!("当前执行路径获取失败：{}", e)), None))?;

    if let Some(exe_dir) = exe_path.parent() {
        return Ok(CustomResult::success(
            None,
            Some(json!({"path": exe_dir.display().to_string()})),
        ));
    } else {
        return Err(CustomResult::error(
            Some(format!("无法获取可执行文件的父目录")),
            None,
        ));
    }
}

#[tauri::command]
pub async fn get_app_version(app_handle: tauri::AppHandle) -> Result<CustomResult, CustomResult> {
    let package_info = app_handle.package_info();
    let version = package_info.version.to_string();

    Ok(CustomResult::success(
        None,
        Some(json!({"version": version})),
    ))
}

fn encode_audio_to_base64(file_path: &str) -> Result<String, CustomResult> {
    let audio_bytes = std::fs::read(file_path)
        .map_err(|e| CustomResult::error(Some(format!("读取音频文件失败：{}", e)), None))?;

    let encoded = general_purpose::STANDARD.encode(audio_bytes);

    Ok(encoded)
}
