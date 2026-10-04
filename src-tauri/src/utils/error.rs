use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("URL 校验失败: {0}")]
    UrlValidationError(String),

    #[error("网络请求失败: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("B 站 API 返回错误: code={code}, message={message}")]
    BiliApiError { code: i32, message: String },

    #[error("视频解析失败: {0}")]
    ParseError(String),

    #[error("下载失败: {0}")]
    DownloadError(String),

    #[error("Sidecar 进程错误: {0}")]
    SidecarError(String),

    #[error("任务未找到: {0}")]
    TaskNotFound(String),

    #[error("路径不安全: {0}")]
    PathNotSafe(String),

    #[error("参数无效: {0}")]
    InvalidArgument(String),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;