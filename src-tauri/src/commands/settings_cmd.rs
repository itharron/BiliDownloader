use crate::models::settings::AppSettings;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tracing::info;

const STORE_FILENAME: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| format!("无法打开配置存储: {}", e))?;

    let value = store
        .get(SETTINGS_KEY)
        .and_then(|v| serde_json::from_value::<AppSettings>(v.clone()).ok());

    Ok(value.unwrap_or_default())
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    info!("保存设置: {:?}", settings);

    let store = app
        .store(STORE_FILENAME)
        .map_err(|e| format!("无法打开配置存储: {}", e))?;

    let value = serde_json::to_value(&settings).map_err(|e| format!("序列化设置失败: {}", e))?;

    store.set(SETTINGS_KEY, value);

    store
        .save()
        .map_err(|e| format!("保存设置失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn select_download_dir() -> Result<String, String> {
    // 由前端通过 dialog 插件处理目录选择
    Ok(String::new())
}