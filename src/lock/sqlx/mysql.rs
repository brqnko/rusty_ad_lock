use crate::Locker;
use crate::lock::LockerGuard;
use crate::lock::Result;

pub struct MySqlLocker;

impl Locker for MySqlLocker {
    type DB = sqlx::mysql::MySql;

    async fn lock<'s, 'p>(
        pool: &sqlx::Pool<Self::DB>,
        text: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<Option<LockerGuard<'s, 'p, Self::DB>>> {
        let timeout = timeout.unwrap_or_default().as_secs();
        let a: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK($1,$2);")
            .bind(text)
            .bind(timeout)
            .fetch_optional(pool)
            .await?;

        todo!()
    }

    async fn unlock(pool: &sqlx::Pool<Self::DB>, text: &str) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_matches, assert_str_eq};
    use std::time::Duration;
    use tokio::time::sleep;

    use super::*;

    use sqlx::MySqlPool;

    #[sqlx::test]
    async fn lock_and_await(pool: MySqlPool) -> sqlx::Result<()> {
        // lock with 64 character with 1 sec
        let guard = MySqlLocker::lock(
            &pool,
            "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
            Some(Duration::from_secs(1)),
        )
        .await;
        assert_matches!(&guard, Ok(Some(_)),);
        assert_str_eq!(
            guard.unwrap().unwrap().text,
            "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB"
        );

        // lock again with same key within 1 sec
        assert_matches!(
            MySqlLocker::lock(
                &pool,
                "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
                Some(Duration::from_secs(1))
            )
            .await,
            Ok(None),
        );

        sleep(Duration::from_secs(1)).await;

        // lock again with same key after 1 sec
        let guard = MySqlLocker::lock(
            &pool,
            "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
            Some(Duration::from_secs(1)),
        )
        .await;
        assert_matches!(&guard, Ok(Some(_)),);
        let guard = guard.unwrap().unwrap();
        assert_str_eq!(
            guard.text,
            "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn empty_lock(pool: MySqlPool) -> sqlx::Result<()> {
        // lock with empty key and 1 sec
        let guard = MySqlLocker::lock(&pool, "", Some(Duration::from_secs(1))).await;
        assert_matches!(&guard, Ok(Some(_)),);
        assert_str_eq!(guard.unwrap().unwrap().text, "");

        // lock again with empty key whicin 1 sec
        let guard = MySqlLocker::lock(&pool, "", None).await;
        assert_matches!(&guard, Ok(None),);

        sleep(Duration::from_secs(1)).await;

        // lock with empty key after 1 sec
        let guard = MySqlLocker::lock(&pool, "", Some(Duration::from_secs(0))).await;
        assert_matches!(&guard, Ok(Some(_)));
        assert_str_eq!(guard.unwrap().unwrap().text, "");

        // lock with empty key instantlly
        let guard = MySqlLocker::lock(&pool, "", Some(Duration::from_secs(0))).await;
        assert_matches!(&guard, Ok(Some(_)));
        assert_str_eq!(guard.unwrap().unwrap().text, "");

        Ok(())
    }

    #[sqlx::test]
    async fn lock_longer_than_64(pool: MySqlPool) -> sqlx::Result<()> {
        let guard = MySqlLocker::lock(
            &pool,
            "QcpCXg6KQ6rPWuU6hYntMNrbQupv31fJTcMjcsrnKnRSKektjCD8QS0ImLfgiuKk1",
            Some(Duration::from_secs(1)),
        )
        .await;

        assert_matches!(&guard, Err(_),);
        Ok(())
    }
}
