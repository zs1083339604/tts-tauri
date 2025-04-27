//! 本文件内代码参考开源项目：https://github.com/rany2/edge-tts/tree/master

use crate::utils::custom_result::CustomResult;
use chrono::Utc;
use hex;
use serde_json::json;
use sha2::{Digest, Sha256};

pub struct TTS {}

impl TTS {
    fn num_to_str(&self, num: &str) -> String {
        if !num.contains('e') {
            return num.to_string();
        }
        let mut num_str = num
            .trim_matches(|c| c == ' ' || c == '"' || c == '=' || c == '\'')
            .to_string();
        num_str.retain(|c| c.is_digit(10));
        let mut result = String::new();
        let mut n = num_str.parse::<i64>().unwrap_or(0);
        while n > 0 {
            let v = n % 10;
            n /= 10;
            result.insert(0, std::char::from_digit(v as u32, 10).unwrap());
        }
        result
    }

    pub fn convert_to_audio_format_websocket_string(&self, output_format: &str) -> String {
        format!(
            "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"}},\"outputFormat\":\"{}\"}}}}}}}}",
            self.date_to_string(),
            output_format
        )
    }

    fn convert_to_ssml_text(
        &self,
        voice: &str,
        text: &str,
        pitch: &str,
        rate: &str,
        volume: &str,
    ) -> String {
        format!(
            "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'><voice name='{}'><prosody pitch='{}Hz' rate ='{}%' volume='{}%'>{}</prosody></voice></speak>",
            voice, pitch, rate, volume, text
        )
    }

    pub fn convert_to_ssml_websocket_string(
        &self,
        request_id: &str,
        voice: &str,
        msg: &str,
        pitch: &str,
        rate: &str,
        volume: &str,
    ) -> String {
        format!(
            "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{}Z\r\nPath:ssml\r\n\r\n{}",
            request_id,
            self.date_to_string(),
            self.convert_to_ssml_text(voice, msg, pitch, rate, volume)
        )
    }

    fn date_to_string(&self) -> String {
        let now = Utc::now();
        now.format("%a %b %d %Y %H:%M:%S GMT+0000 (Coordinated Universal Time)")
            .to_string()
    }

    fn get_unix_timestamp(&self) -> f64 {
        Utc::now().timestamp() as f64
    }

    pub fn generate_sec_ms_gec(&self) -> Result<CustomResult, CustomResult> {
        let mut ticks = self.get_unix_timestamp();
        ticks += 11644473600.0;
        ticks = (ticks / 300.0).floor() * 300.0;
        let ticks_int = (ticks * 1e7) as i64;
        let str_to_hash = format!(
            "{}{}",
            self.num_to_str(&ticks_int.to_string()),
            "6A5AA1D4EAFF4E9FB37E23D68491D6F4"
        );
        let mut hasher = Sha256::new();
        hasher.update(&str_to_hash);
        let result = hasher.finalize();
        let hex_str = hex::encode(result).to_uppercase();

        Ok(CustomResult::success(None, Some(json!({ "hax": hex_str }))))
    }
}
