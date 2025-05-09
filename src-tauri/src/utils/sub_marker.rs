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

        let mut new_cues: Vec<Cue> = Vec::new();
        let mut current_index = 0;
        let mut last_match_end: isize = -1;
        let mut match_start_index = 0;
        let punctuation_re = regex::Regex::new(r#"[。！？？，；,()\[\]（）【】{}、\.\?!;:<>《》「」『』“”‘’"…\n]+"#).map_err(|e| CustomResult::error(Some(e.to_string()), None))?;

        for i in 0..self.cues.len() {
            let cue = &self.cues[i];
            let search_text = &cue.content;

            if let Some(position) = all_text[current_index..].find(search_text) {
                let position = current_index + position;
                if last_match_end == -1 {
                    last_match_end = position as isize;
                }

                let mut next_position = position + search_text.len();
                let mut max_while = 0;
                let mut chars = all_text[next_position..].chars();

                // 如果是英文，这里有时会变成空格，需要过滤一下
                while let Some(next_char) = chars.next() {
                    // 如果不是空格 或 超过了最大限制则退出
                    if next_char != ' ' || max_while == 10 {
                        break;
                    }
                    next_position += next_char.len_utf8();
                    max_while += 1;
                }

                // 判断下一个字符是否是标点
                let next_char = all_text[next_position..].chars().next().unwrap_or(' ');
                if punctuation_re.is_match(&next_char.to_string()) {
                    let start = self.cues[match_start_index].start.clone();
                    let end_pos = next_position;
                    let extracted_text = &all_text[last_match_end as usize..end_pos];
                    new_cues.push(Cue {
                        index: cue.index,
                        start,
                        end: cue.end.clone(),
                        content: extracted_text.to_string(),
                    });
                    last_match_end = -1;
                    match_start_index = i + 1;
                }

                current_index = position;
            }
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
