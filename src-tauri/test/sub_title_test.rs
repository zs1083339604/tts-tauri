// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use utils::sub_marker::generate_srt;
pub mod utils;

use std::fs;
use std::io;
use serde_json::{from_str, Value};

fn main() -> io::Result<()> {
    // tts_tauri_lib::run()
    let json_path = "test/复杂文本2.json";
    let json_content = fs::read_to_string(json_path)?;
    let mut messages: Vec<Value> = from_str::<Vec<Value>>(&json_content)?.to_vec();

    let txt_path = "test/复杂文本2.txt";
    let txt_content = fs::read_to_string(txt_path)?;
    
    let result = generate_srt(&messages, "test/output.srt", "mergeByNumber", 5, Some(txt_content.as_str()));

    match result {
        Ok(_) => print!("成功"),
        Err(_) => print!("失败")
    };

    Ok(())
}
