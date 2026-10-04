use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

static AUTH_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build auth HTTP client")
});

#[derive(Debug, Deserialize)]
struct BiliApiResponse<T> {
    code: i32,
    message: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct QrGenerateData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrGenerateResult {
    pub qr_url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrPollResult {
    pub status: QrStatus,
    pub message: String,
    pub sessdata: Option<String>,
    pub bili_jct: Option<String>,
    pub dedeuserid: Option<String>,
    pub uname: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QrStatus {
    Pending,
    Scanned,
    Confirmed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliLoginInfo {
    pub sessdata: String,
    pub bili_jct: String,
    pub dedeuserid: String,
    pub uname: String,
    pub avatar: String,
}

pub async fn generate_qr() -> Result<QrGenerateResult> {
    info!("生成登录二维码...");

    let resp = AUTH_CLIENT
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await?
        .json::<BiliApiResponse<QrGenerateData>>()
        .await?;

    if resp.code != 0 {
        let msg = resp.message.unwrap_or_default();
        warn!("生成二维码失败: code={}, msg={}", resp.code, msg);
        anyhow::bail!("生成二维码失败: {}", msg);
    }

    let data = resp.data.ok_or_else(|| anyhow::anyhow!("二维码数据为空"))?;

    info!(
        "二维码已生成: qrcode_key={}",
        data.qrcode_key
    );

    Ok(QrGenerateResult {
        qr_url: data.url,
        qrcode_key: data.qrcode_key,
    })
}

pub async fn poll_qr(qrcode_key: &str) -> Result<QrPollResult> {
    let resp = AUTH_CLIENT
        .get(&format!(
            "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}",
            qrcode_key
        ))
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await?;

    let status_code = resp.status();

    // 使用 get_all 处理多个 set-cookie 头
    let mut sessdata = None;
    let mut bili_jct = None;
    let mut dedeuserid = None;

    for cookie_value in resp.headers().get_all("set-cookie") {
        let cookie_str = cookie_value.to_str().unwrap_or("");
        for part in cookie_str.split("; ") {
            if part.starts_with("SESSDATA=") {
                let raw = part.strip_prefix("SESSDATA=").unwrap_or("").to_string();
                // SESSDATA 可能含 URL 编码字符（如 %2C），解码后存储
                sessdata = Some(
                    urlencoding::decode(&raw)
                        .map(|s| s.into_owned())
                        .unwrap_or(raw),
                );
            } else if part.starts_with("bili_jct=") {
                bili_jct = Some(
                    part.strip_prefix("bili_jct=")
                        .unwrap_or("")
                        .to_string(),
                );
            } else if part.starts_with("DedeUserID=") {
                dedeuserid = Some(
                    part.strip_prefix("DedeUserID=")
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }
    }

    let body = resp.text().await?;

    info!(
        "轮询二维码状态: qrcode_key={}, http_status={}, sessdata_len={}",
        qrcode_key,
        status_code,
        sessdata.as_ref().map(|s| s.len()).unwrap_or(0),
    );

    // B站 poll API 真实返回码：
    //   0      → 确认成功
    //   86101  → 未扫码
    //   86090  → 已扫码，等待确认
    //   86038  → 二维码过期
    #[derive(Deserialize)]
    struct PollData {
        code: i32,
        #[serde(default)]
        message: String,
        #[allow(dead_code)]
        url: Option<String>,
        #[serde(rename = "refresh_token")]
        #[allow(dead_code)]
        refresh_token: Option<String>,
        #[allow(dead_code)]
        timestamp: Option<i64>,
    }

    match serde_json::from_str::<PollData>(&body) {
        Ok(data) => {
            info!("轮询响应 code={}, message={}", data.code, data.message);
            match data.code {
                0 => {
                    // 用户已在手机上确认登录
                    if let (Some(ref sd), Some(ref ct)) = (&sessdata, &bili_jct) {
                        info!(
                            "扫码确认成功! sessdata={}..., bili_jct={}...",
                            &sd[..sd.len().min(8)],
                            &ct[..ct.len().min(8)]
                        );
                        Ok(QrPollResult {
                            status: QrStatus::Confirmed,
                            message: "扫码成功".to_string(),
                            sessdata,
                            bili_jct,
                            dedeuserid,
                            uname: None,
                        })
                    } else {
                        // code=0 但 cookie 不完整，但仍算确认成功
                        warn!("扫码确认成功但 Cookie 不完整, 仍标记为已确认");
                        Ok(QrPollResult {
                            status: QrStatus::Confirmed,
                            message: "扫码成功（Cookie不完整）".to_string(),
                            sessdata,
                            bili_jct,
                            dedeuserid,
                            uname: None,
                        })
                    }
                }
                86101 => {
                    // 未扫码
                    Ok(QrPollResult {
                        status: QrStatus::Pending,
                        message: "等待扫码".to_string(),
                        sessdata: None,
                        bili_jct: None,
                        dedeuserid: None,
                        uname: None,
                    })
                }
                86090 => {
                    // 已扫码，等待用户在手机上确认
                    info!("已扫码，等待用户确认...");
                    Ok(QrPollResult {
                        status: QrStatus::Scanned,
                        message: "已扫码，请在手机上确认".to_string(),
                        sessdata: None,
                        bili_jct: None,
                        dedeuserid: None,
                        uname: None,
                    })
                }
                86038 => {
                    warn!("二维码已过期");
                    Ok(QrPollResult {
                        status: QrStatus::Expired,
                        message: "二维码已过期，请重新生成".to_string(),
                        sessdata: None,
                        bili_jct: None,
                        dedeuserid: None,
                        uname: None,
                    })
                }
                other => {
                    warn!("轮询返回未知 code: {}, message={}", other, data.message);
                    // 未知码视为 continue polling（Pending）
                    Ok(QrPollResult {
                        status: QrStatus::Pending,
                        message: data.message,
                        sessdata: None,
                        bili_jct: None,
                        dedeuserid: None,
                        uname: None,
                    })
                }
            }
        }
        Err(parse_err) => {
            // JSON 解析失败 → 只看 HTTP 状态码决定
            warn!(
                "poll 响应非 JSON, http_status={}, body_preview={}",
                status_code,
                &body[..body.len().min(300)]
            );
            if status_code.is_success() && sessdata.is_some() {
                // HTTP 200 且有 SESSDATA → 可能是重定向后登录成功
                Ok(QrPollResult {
                    status: QrStatus::Confirmed,
                    message: "登录成功".to_string(),
                    sessdata,
                    bili_jct,
                    dedeuserid,
                    uname: None,
                })
            } else {
                anyhow::bail!(
                    "解析登录响应失败: {} (status={})",
                    parse_err,
                    status_code
                )
            }
        }
    }
}

pub async fn fetch_user_info(sessdata: &str) -> Result<BiliLoginInfo> {
    info!("获取用户信息...");

    #[derive(Deserialize)]
    struct NavData {
        #[serde(rename = "isLogin")]
        is_login: bool,
        #[serde(default)]
        uname: String,
        #[serde(default)]
        face: String,
        #[serde(default)]
        mid: u64,
    }

    let resp = AUTH_CLIENT
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("Referer", "https://www.bilibili.com")
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .await?
        .json::<BiliApiResponse<NavData>>()
        .await?;

    if resp.code != 0 {
        let msg = resp.message.unwrap_or_default();
        anyhow::bail!("获取用户信息失败: {}", msg);
    }

    let data = resp
        .data
        .ok_or_else(|| anyhow::anyhow!("用户数据为空"))?;

    if !data.is_login {
        anyhow::bail!("未登录或登录已过期");
    }

    info!("用户信息获取成功: {} (mid={})", data.uname, data.mid);

    Ok(BiliLoginInfo {
        sessdata: sessdata.to_string(),
        bili_jct: String::new(),
        dedeuserid: data.mid.to_string(),
        uname: data.uname,
        avatar: data.face,
    })
}