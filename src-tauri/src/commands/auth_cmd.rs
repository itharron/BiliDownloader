use crate::auth::{self, BiliLoginInfo, QrGenerateResult, QrPollResult};
use crate::models::settings::AppSettings;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;
use tracing::info;

const STORE_FILENAME: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";

#[tauri::command]
pub async fn generate_qr() -> Result<QrGenerateResult, String> {
    auth::generate_qr()
        .await
        .map_err(|e| format!("生成二维码失败: {}", e))
}

#[tauri::command]
pub async fn poll_qr(qrcode_key: String) -> Result<QrPollResult, String> {
    auth::poll_qr(&qrcode_key)
        .await
        .map_err(|e| format!("轮询二维码状态失败: {}", e))
}

#[tauri::command]
pub async fn confirm_login(
    app: AppHandle,
    sessdata: String,
    bili_jct: String,
    dedeuserid: String,
) -> Result<BiliLoginInfo, String> {
    info!("确认登录: dedeuserid={}", dedeuserid);

    let user_info = auth::fetch_user_info(&sessdata)
        .await
        .map_err(|e| format!("获取用户信息失败: {}", e))?;

    // 保存登录态到设置
    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| format!("无法打开配置存储: {}", e))?;

    let mut settings: AppSettings = if let Some(value) = store.get(SETTINGS_KEY) {
        serde_json::from_value(value.clone()).unwrap_or_default()
    } else {
        AppSettings::default()
    };

    settings.cookie_sessdata = Some(sessdata.clone());
    settings.cookie_bili_jct = Some(bili_jct.clone());
    settings.cookie_dedeuserid = Some(dedeuserid.clone());
    settings.login_uname = Some(user_info.uname.clone());
    settings.login_avatar = Some(user_info.avatar.clone());

    let new_value =
        serde_json::to_value(&settings).map_err(|e| format!("序列化设置失败: {}", e))?;
    store.set(SETTINGS_KEY, new_value);
    store.save().map_err(|e| format!("保存设置失败: {}", e))?;

    Ok(user_info)
}

#[tauri::command]
pub async fn logout(app: AppHandle) -> Result<(), String> {
    info!("退出登录");

    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| format!("无法打开配置存储: {}", e))?;

    let mut settings: AppSettings = if let Some(value) = store.get(SETTINGS_KEY) {
        serde_json::from_value(value.clone()).unwrap_or_default()
    } else {
        return Ok(())
    };
    settings.cookie_sessdata = None;
    settings.cookie_bili_jct = None;
    settings.cookie_dedeuserid = None;
    settings.login_uname = None;
    settings.login_avatar = None;

    let new_value =
        serde_json::to_value(&settings).map_err(|e| format!("序列化设置失败: {}", e))?;
    store.set(SETTINGS_KEY, new_value);
    store.save().map_err(|e| format!("保存设置失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_login_status(app: AppHandle) -> Result<Option<BiliLoginInfo>, String> {
    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| format!("无法打开配置存储: {}", e))?;

    if let Some(value) = store.get(SETTINGS_KEY) {
        let settings: AppSettings =
            serde_json::from_value(value.clone()).unwrap_or_default();
        if let Some(sessdata) = &settings.cookie_sessdata {
            if !sessdata.is_empty() {
                return Ok(Some(BiliLoginInfo {
                    sessdata: sessdata.clone(),
                    bili_jct: settings.cookie_bili_jct.clone().unwrap_or_default(),
                    dedeuserid: settings.cookie_dedeuserid.clone().unwrap_or_default(),
                    uname: settings.login_uname.clone().unwrap_or_default(),
                    avatar: settings.login_avatar.clone().unwrap_or_default(),
                }));
            }
        }
    }

    Ok(None)
}

#[tauri::command]
pub fn get_data_dir(app: AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取数据目录: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}