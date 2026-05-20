use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(#[from] dict_db::DbError),
    #[error("online error: {0}")]
    Online(String),
    #[error("{0}")]
    Message(String),
}
