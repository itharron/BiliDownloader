use crate::utils::crypto::get_mixin_key;
use anyhow::Result;
use serde::Deserialize;

const WBI_KEY_URL: &str = "https://api.bilibili.com/x/web-interface/nav";

#[derive(Debug, Deserialize)]
struct NavResponse {
    code: i32,
    data: Option<NavData>,
}

#[derive(Debug, Deserialize)]
struct NavData {
    wbi_img: Option<WbiImg>,
}

#[derive(Debug, Deserialize)]
struct WbiImg {
    img_url: String,
    sub_url: String,
}

/// 获取 WBI 签名需要的 mixin_key
pub async fn fetch_wbi_key(client: &reqwest::Client) -> Result<String> {
    let resp: NavResponse = client
        .get(WBI_KEY_URL)
        .header("Referer", "https://www.bilibili.com")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await?
        .json()
        .await?;

    if resp.code != 0 {
        anyhow::bail!("获取 WBI key 失败: code={}", resp.code);
    }

    let wbi = resp
        .data
        .and_then(|d| d.wbi_img)
        .ok_or_else(|| anyhow::anyhow!("wbi_img 字段缺失"))?;

    let img_key = extract_key_from_url(&wbi.img_url);
    let sub_key = extract_key_from_url(&wbi.sub_url);
    let raw_key = format!("{}{}", img_key, sub_key);

    Ok(get_mixin_key(&raw_key))
}

fn extract_key_from_url(url: &str) -> String {
    url.split('/')
        .last()
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_key() {
        assert_eq!(
            extract_key_from_url("https://i0.hdslb.com/bfs/wbi/abc123.png"),
            "abc123"
        );
    }
}