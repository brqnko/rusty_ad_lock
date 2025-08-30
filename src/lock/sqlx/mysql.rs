use crate::Locker;
use crate::lock::LockerGuard;
use crate::lock::Result;

pub struct MySqlLocker;

impl Locker for MySqlLocker {
    type Pool = sqlx::Pool<sqlx::mysql::MySql>;

    fn lock(
        pool: &Self::Pool,
        text: &str,
        timeout: Option<std::time::Duration>,
    ) -> impl Future<Output = Result<Option<LockerGuard>>> + Send {
        async move {
            todo!()
        }
    }

    fn unlock(pool: &Self::Pool, text: &str) -> impl Future<Output = Result<()>> {
        async move {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {}
