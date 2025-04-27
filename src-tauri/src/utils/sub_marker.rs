use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;

use super::custom_result::CustomResult;

#[derive(Debug, Clone)]
struct Cue {
    index: usize,
    start: String,
    end: String,
    content: String,
}

pub struct SubMaker {
    cues: Vec<Cue>,
}

impl SubMaker {
    pub fn new() -> Self {
        SubMaker { cues: Vec::new() }
    }

    pub fn feed(&mut self, msg: &serde_json::Value) -> Result<CustomResult, CustomResult> {
        if msg["Type"] != "WordBoundary" {
            return Err(CustomResult::error(
                Some("错误的Message类型，仅支持'WordBoundary'".to_string()),
                None,
            ));
        }

        let offset = msg["Data"]["Offset"].as_u64().unwrap();
        let duration = msg["Data"]["Duration"].as_u64().unwrap();
        let text = msg["Data"]["text"]["Text"].as_str().unwrap().to_string();

        let start = self.microseconds_to_time(offset);
        let end = self.microseconds_to_time(offset + duration);

        self.cues.push(Cue {
            index: self.cues.len() + 1,
            start,
            end,
            content: text,
        });

        Ok(CustomResult::success(None, None))
    }

    pub fn merge_by_number(&mut self, words: i32) -> Result<CustomResult, CustomResult> {
        if words == 0 || self.cues.is_empty() {
            return Ok(CustomResult::success(None, None));
        }

        let mut new_cues = Vec::new();
        let mut current_cue = self.cues[0].clone();

        for cue in self.cues.iter().skip(1) {
            let word_count = current_cue.content.split_whitespace().count();

            if word_count < words.try_into().unwrap() {
                current_cue.end = cue.end.clone();
                current_cue.content = format!("{} {}", current_cue.content, cue.content);
            } else {
                new_cues.push(current_cue);
                current_cue = cue.clone();
            }
        }

        new_cues.push(current_cue);
        self.cues = new_cues;
        self.reset_index();

        Ok(CustomResult::success(None, None))
    }

    pub fn merge_by_punctuation(&mut self, all_text: &str) -> Result<CustomResult, CustomResult> {
        if all_text.is_empty() {
            return Err(CustomResult::error(
                Some("你选择了标点分句，但参考句子是空的".to_string()),
                None,
            ));
        }

        let rule = regex::Regex::new(
            r#"[。！？？，；,()\[\]（）【】{}、\.\?!;:<>《》「」『』“”‘’"—…\-\n]+"#,
        )
        .map_err(|e| CustomResult::error(Some(e.to_string()), None))?;
        let sentences: Vec<&str> = rule
            .split(all_text)
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut current_sentence_index = 0;
        let mut current_cue_index = 0;
        let mut current_sentence = sentences.get(current_sentence_index).copied();
        let mut new_cues = Vec::new();
        let mut current_cue: Option<Cue> = None;

        for cue in &self.cues {
            current_cue_index += 1;

            if current_cue.is_none() {
                current_cue = Some(cue.clone());
            } else {
                let mut unwrapped_cue = current_cue.unwrap();
                unwrapped_cue.end = cue.end.clone();
                unwrapped_cue.content.push_str(&cue.content);
                current_cue = Some(unwrapped_cue);
            }

            // 判断当前和下一个字幕是否为英文、数字
            let is_alphanumeric = regex::Regex::new(r"^[a-zA-Z0-9\s]+$")
                .map_err(|e| CustomResult::error(Some(e.to_string()), None))?;
            let next_cue_content = self.cues.get(current_cue_index).map(|c| &c.content);
            if is_alphanumeric.is_match(&cue.content)
                && next_cue_content.map_or(false, |c| is_alphanumeric.is_match(c))
            {
                if let Some(ref mut c) = current_cue {
                    c.content.push(' ');
                }
            }

            // 如果当前字幕内容包含当前句子的内容，则完成一个句子
            if let (Some(ref mut c), Some(sentence)) = (&mut current_cue, current_sentence) {
                if c.content
                    .replace(" ", "")
                    .contains(&sentence.replace(" ", ""))
                {
                    new_cues.push(c.clone());
                    current_cue = None;
                    current_sentence_index += 1;
                    current_sentence = sentences.get(current_sentence_index).copied();
                }
            }
        }

        if let Some(cue) = current_cue {
            new_cues.push(cue);
        }

        self.cues = new_cues;
        self.reset_index();

        Ok(CustomResult::success(None, None))
    }

    pub fn get_srt(&self) -> String {
        self.cues
            .iter()
            .map(|cue| {
                format!(
                    "{}\n{} --> {}\n{}\n",
                    cue.index,
                    cue.start,
                    cue.end,
                    cue.content.trim()
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn reset_index(&mut self) {
        for (i, cue) in self.cues.iter_mut().enumerate() {
            cue.index = i + 1;
        }
    }

    fn microseconds_to_time(&self, microseconds: u64) -> String {
        let milliseconds = (microseconds / 10000) as u64;
        let hours = milliseconds / 3600000;
        let minutes = (milliseconds % 3600000) / 60000;
        let seconds = (milliseconds % 60000) / 1000;
        let ms = milliseconds % 1000;
        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, ms)
    }
}

// 写入 SRT 文件
pub fn generate_srt(
    messages: &[serde_json::Value],
    srt_path: &str,
    option: &str,
    number: i32,
    all_text: Option<&str>,
) -> Result<CustomResult, CustomResult> {
    let mut maker = SubMaker::new();

    for msg in messages {
        if let Some(meta) = msg.get("Metadata").and_then(|v| v.get(0)) {
            if meta["Type"] == "WordBoundary" {
                let _ = maker.feed(meta)?;
            }
        }
    }

    match option {
        "mergeByPunctuation" => {
            let _ = maker.merge_by_punctuation(all_text.unwrap_or(""))?;
        }
        "mergeByNumber" => {
            let _ = maker.merge_by_number(number)?;
        }
        _ => {}
    }

    let srt_content = maker.get_srt();
    let mut file =
        File::create(srt_path).map_err(|e| CustomResult::error(Some(e.to_string()), None))?;
    file.write_all(srt_content.as_bytes())
        .map_err(|e| CustomResult::error(Some(e.to_string()), None))?;
    Ok(CustomResult::success(None, None))
}
