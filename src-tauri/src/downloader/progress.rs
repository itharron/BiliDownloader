use crate::models::task::DownloadProgress;

/// 解析 sidecar 输出中的进度信息
pub struct ProgressParser;

impl ProgressParser {
    /// 解析 yt-dlp 的进度输出行
    /// 格式示例: [download]  10.5% of 100.00MiB at 5.00MiB/s ETA 00:18
    pub fn parse_ytdlp_line(line: &str) -> Option<DownloadProgress> {
        if !line.starts_with("[download]") {
            return None;
        }

        let content = line.trim_start_matches("[download]").trim();

        let percentage = Self::extract_percentage(content)?;
        let (downloaded_bytes, total_bytes) = Self::extract_bytes(content);
        let speed_bytes_per_sec = Self::extract_speed(content);
        let eta_seconds = Self::extract_eta(content);

        Some(DownloadProgress {
            percentage,
            downloaded_bytes,
            total_bytes,
            speed_bytes_per_sec,
            eta_seconds,
        })
    }

    fn extract_percentage(content: &str) -> Option<f32> {
        let percent_str = content.split('%').next()?.trim();
        percent_str.parse::<f32>().ok()
    }

    fn extract_bytes(content: &str) -> (u64, u64) {
        let parts: Vec<&str> = content.split_whitespace().collect();

        let mut downloaded = 0u64;
        let mut total = 0u64;

        for (i, part) in parts.iter().enumerate() {
            if *part == "of" && i + 1 < parts.len() {
                if i > 0 {
                    downloaded = parse_size_str(parts[i - 1]);
                }
                total = parse_size_str(parts[i + 1]);
            }
        }

        (downloaded, total)
    }

    fn extract_speed(content: &str) -> u64 {
        let parts: Vec<&str> = content.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "at" && i + 1 < parts.len() {
                return parse_size_str(parts[i + 1]);
            }
        }
        0
    }

    fn extract_eta(content: &str) -> i64 {
        let parts: Vec<&str> = content.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "ETA" && i + 1 < parts.len() {
                let time_str = parts[i + 1];
                let segments: Vec<&str> = time_str.split(':').collect();
                if segments.len() == 2 {
                    if let (Ok(minutes), Ok(seconds)) =
                        (segments[0].parse::<i64>(), segments[1].parse::<i64>())
                    {
                        return minutes * 60 + seconds;
                    }
                }
                if segments.len() == 3 {
                    if let (Ok(h), Ok(m), Ok(s)) = (
                        segments[0].parse::<i64>(),
                        segments[1].parse::<i64>(),
                        segments[2].parse::<i64>(),
                    ) {
                        return h * 3600 + m * 60 + s;
                    }
                }
            }
        }
        0
    }
}

/// 将大小字符串（如 100.00MiB, 5.00GiB）转为字节
fn parse_size_str(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }

    let num_str: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    let num: f64 = num_str.parse().unwrap_or(0.0);
    let unit = s[num_str.len()..]
        .to_lowercase()
        .trim_end_matches("/s")
        .to_string();

    match unit.as_str() {
        "kib" | "kb" => (num * 1024.0) as u64,
        "mib" | "mb" => (num * 1024.0 * 1024.0) as u64,
        "gib" | "gb" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
        "tib" | "tb" => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
        _ => num as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress_line() {
        let line = "[download]  10.5% of 100.00MiB at 5.00MiB/s ETA 00:18";
        let progress = ProgressParser::parse_ytdlp_line(line).unwrap();
        assert!((progress.percentage - 10.5).abs() < 0.01);
        assert_eq!(progress.speed_bytes_per_sec, 5 * 1024 * 1024);
        assert_eq!(progress.eta_seconds, 18);
    }

    #[test]
    fn test_parse_size_str() {
        assert_eq!(parse_size_str("100.00MiB"), 100 * 1024 * 1024);
        assert_eq!(parse_size_str("5.00GiB"), 5 * 1024 * 1024 * 1024);
    }
}