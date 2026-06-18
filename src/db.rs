use std::path::PathBuf;

use crate::portfolio::{PortfolioImportError, SqlitePortfolioRepository};

/// Opens (or creates) the on-disk SQLite database that stores portfolio holdings.
///
/// The file lives at `{data_dir}/meroalpha/portfolio.db`, where `data_dir` is
/// the platform-appropriate application data directory:
///   - macOS:   ~/Library/Application Support/meroalpha/portfolio.db
///   - Linux:   ~/.local/share/meroalpha/portfolio.db
///   - Windows: %APPDATA%\meroalpha\portfolio.db
///
/// Falls back to the current working directory if the platform data dir is unavailable.
pub fn open_app_db() -> Result<SqlitePortfolioRepository, PortfolioImportError> {
    let db_path = app_db_path();

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| PortfolioImportError::Storage(format!("create db directory: {e}")))?;
    }

    let connection = rusqlite::Connection::open(&db_path).map_err(|e| {
        PortfolioImportError::Storage(format!("open sqlite at {}: {e}", db_path.display()))
    })?;

    SqlitePortfolioRepository::from_connection(connection)
}

fn app_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meroalpha")
        .join("portfolio.db")
}
