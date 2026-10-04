pub mod auth;
pub mod commands;
pub mod downloader;
pub mod models;
pub mod parser;
pub mod utils;

use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::logs::init_logging();

    info!("BiliDownloader starting...");

    let task_manager_state = commands::task_cmd::TaskManagerState::new();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(task_manager_state)
        .invoke_handler(tauri::generate_handler![
            commands::auth_cmd::generate_qr,
            commands::auth_cmd::poll_qr,
            commands::auth_cmd::confirm_login,
            commands::auth_cmd::logout,
            commands::auth_cmd::get_login_status,
            commands::auth_cmd::get_data_dir,
            commands::parse_cmd::parse_video_url,
            commands::task_cmd::start_download,
            commands::task_cmd::pause_download,
            commands::task_cmd::resume_download,
            commands::task_cmd::cancel_download,
            commands::task_cmd::retry_download,
            commands::task_cmd::list_tasks,
            commands::task_cmd::load_task_history,
            commands::task_cmd::open_output_folder,
            commands::task_cmd::delete_download,
            commands::settings_cmd::get_settings,
            commands::settings_cmd::save_settings,
            commands::settings_cmd::select_download_dir,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running BiliDownloader");
}