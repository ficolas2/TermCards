use std::io;

use directories::ProjectDirs;
use sqlx::{
    Pool, SqlitePool,
    migrate::{MigrateDatabase, MigrateError},
};
use thiserror::Error;

pub struct Repository {
    pub(in crate::repository) pool: Pool<sqlx::Sqlite>,
}

#[derive(Debug, Error)]
pub enum CreateRepositoryError {
    #[error("failed to create directory: {0}")]
    Io(#[from] io::Error),

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("home directory not found")]
    NoHomeDir,

    #[error("migrate error: {0}")]
    MigrateError(#[from] MigrateError),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl Repository {
    pub async fn new() -> Result<Repository, CreateRepositoryError> {
        let mut path = ProjectDirs::from("com", "ficolas2", "termcards")
            .ok_or_else(|| CreateRepositoryError::NoHomeDir)?
            .data_local_dir()
            .to_path_buf();
        path.push("termcards.sqlite");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite://{}", path.display());
        if !sqlx::Sqlite::database_exists(&db_url)
            .await
            .unwrap_or(false)
        {
            sqlx::Sqlite::create_database(&db_url).await?;
        }
        let pool = SqlitePool::connect(&db_url).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Repository { pool })
    }
}
