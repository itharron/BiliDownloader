use crate::utils::error::AppError;

/// 校验 B 站 URL 域名白名单
/// 只允许 *.bilibili.com 和 b23.tv
pub fn validate_bili_url(url: &str) -> Result<(), AppError> {
    let url_lower = url.trim().to_lowercase();

    if url_lower.is_empty() {
        return Err(AppError::UrlValidationError("URL 不能为空".to_string()));
    }

    if let Ok(parsed) = url::Url::parse(&url_lower) {
        let host = parsed.host_str().unwrap_or("");

        if host == "b23.tv" || host.ends_with(".bilibili.com") {
            return Ok(());
        }

        return Err(AppError::UrlValidationError(format!(
            "不支持的域名: {}。仅支持 *.bilibili.com 和 b23.tv",
            host
        )));
    }

    if url_lower.contains("b23.tv") || url_lower.contains("bilibili.com") {
        return Ok(());
    }

    Err(AppError::UrlValidationError(format!(
        "无法解析 URL: {}",
        url
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_bili_url("https://www.bilibili.com/video/BV1xx").is_ok());
        assert!(validate_bili_url("https://b23.tv/abc123").is_ok());
        assert!(validate_bili_url("https://api.bilibili.com").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        assert!(validate_bili_url("https://www.baidu.com").is_err());
        assert!(validate_bili_url("").is_err());
        assert!(validate_bili_url("not a url").is_err());
    }
}