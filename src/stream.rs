use crate::prelude::*;

pub type Update<T> = Option<T>;
pub type UpdateResult<T, E> = Result<Update<T>, E>;

#[async_trait]
pub trait UpdateStream {
    type Item;
    type Error;
    /// Check if update is available.
    /// If any error during update checking is encountered,
    /// then `Update::Err(T)` should be returned.
    /// Otherwise, return value should be wrapped in `Update::Ok(Update<T>)`
    ///
    /// If new update is found, `Update::Some(T)` should be the wrapped value,
    /// otherwise `Update::None` indicates no update.
    async fn fetch_update(&mut self) -> UpdateResult<Self::Item, Self::Error>;
}
