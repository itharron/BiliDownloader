use crate::models::video_info::{PageInfo, QualityInfo, VideoInfo};
use anyhow::Result;
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{info, warn};

static BILI_REGEX: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r"BV[a-zA-Z0-9]{10}").unwrap());

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .tcp_keepalive(Duration::from_secs(30))
        .pool_max_idle_per_host(2)
        .build()
        .expect("Failed to build HTTP client")
});

pub async fn parse_bilibili_video(url: &str, cookie_sessdata: Option<&str>) -> Result<VideoInfo> {
    let bvid = extract_bvid(url)?;
    info!("解析视频 BV={}, with_cookie={}", bvid, cookie_sessdata.is_some());

    let info_url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid);

    let mut req = HTTP_CLIENT
        .get(&info_url)
        .header("Referer", "https://www.bilibili.com");

    if let Some(sessdata) = cookie_sessdata {
        req = req.header("Cookie", format!("SESSDATA={}", sessdata));
    }

    let resp: BiliApiResponse<VideoData> = req.send().await?.json().await?;

    if resp.code != 0 {
        warn!("B站API返回失败: code={}, message={}", resp.code, resp.message);
        anyhow::bail!("B站API错误: code={}, message={}", resp.code, resp.message);
    }

    let data = resp.data.ok_or_else(|| anyhow::anyhow!("视频数据为空"))?;

    let duration_secs = data.duration;
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;

    let qualities = build_quality_list(&data);
    let pages: Vec<PageInfo> = data
        .pages
        .iter()
        .map(|p| PageInfo {
            cid: p.cid,
            page: p.page,
            title: p.part.clone(),
            duration: p.duration,
        })
        .collect();
    let has_subtitle = data.subtitle.allow_submit;

    info!(
        "解析成功: title={}, up={}, duration={}s, qualities={}",
        data.title,
        data.owner.name,
        duration_secs,
        qualities.len()
    );

    Ok(VideoInfo {
        bvid: data.bvid,
        aid: data.aid,
        cid: data.cid,
        title: data.title,
        up_name: data.owner.name,
        up_avatar: data.owner.face,
        cover_url: data.pic,
        duration: duration_secs,
        duration_text: format!("{:02}:{:02}", minutes, seconds),
        view_count: data.stat.view,
        danmaku_count: data.stat.danmaku,
        qualities,
        pages,
        has_subtitle,
    })
}

fn extract_bvid(url: &str) -> Result<String> {
    BILI_REGEX
        .find(url)
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("未能在 URL 中提取到 BV 号: {}", url))
}

fn build_quality_list(data: &VideoData) -> Vec<QualityInfo> {
    if !data.accept_description.is_empty() {
        return data
            .accept_description
            .iter()
            .zip(data.accept_quality.iter())
            .map(|(desc, id)| QualityInfo {
                id: *id as u32,
                name: quality_id_to_name(*id as u32),
                description: desc.clone(),
                width: 0,
                height: 0,
                codecs: String::new(),
            })
            .collect();
    }

    // 回退：API 未返回清晰度列表时，生成默认基础清晰度
    info!("API 未返回清晰度列表，使用默认基础清晰度");
    let defaults: [(u32, &str); 4] = [
        (80, "高清 1080P"),
        (64, "高清 720P"),
        (32, "清晰 480P"),
        (16, "流畅 360P"),
    ];
    defaults
        .iter()
        .map(|(id, desc)| QualityInfo {
            id: *id,
            name: quality_id_to_name(*id),
            description: desc.to_string(),
            width: 0,
            height: 0,
            codecs: String::new(),
        })
        .collect()
}

fn quality_id_to_name(id: u32) -> String {
    match id {
        127 => "超高清 8K",
        126 => "杜比视界",
        125 => "HDR 真彩",
        120 => "超清 4K",
        116 => "高清 1080P60",
        112 => "高清 1080P+",
        80 => "高清 1080P",
        74 => "高清 720P60",
        64 => "高清 720P",
        32 => "清晰 480P",
        16 => "流畅 360P",
        other => return format!("未知({})", other),
    }
    .to_string()
}

// ---------- B 站 API 返回结构 ----------

#[derive(Debug, Deserialize)]
struct BiliApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct VideoData {
    bvid: String,
    aid: u64,
    cid: u64,
    title: String,
    pic: String,
    duration: u64,
    owner: Owner,
    stat: Stat,
    pages: Vec<PageData>,
    #[serde(default)]
    accept_description: Vec<String>,
    #[serde(default)]
    accept_quality: Vec<i64>,
    subtitle: SubtitleInfo,
}

#[derive(Debug, Deserialize)]
struct Owner {
    name: String,
    face: String,
}

#[derive(Debug, Deserialize)]
struct Stat {
    view: u64,
    danmaku: u64,
}

#[derive(Debug, Deserialize)]
struct PageData {
    cid: u64,
    page: u32,
    part: String,
    duration: u64,
}

#[derive(Debug, Deserialize, Default)]
struct SubtitleInfo {
    #[serde(default)]
    allow_submit: bool,
}