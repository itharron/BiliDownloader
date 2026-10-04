use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(
                "bili_downloader=debug,\
                 tauri=info,\
                 warn",
            )
        });

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(filter)
        .init();

    tracing::info!("日志系统初始化完成");
}