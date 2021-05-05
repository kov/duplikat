use thiserror::Error;

#[derive(Error, Debug)]
pub enum DuplikatError {
    #[error("GLib Error: {0}")]
    GLibError(#[from] glib::error::Error),

    #[error("Input/Output error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
