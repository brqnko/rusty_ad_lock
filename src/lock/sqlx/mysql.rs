use crate::{Error, Locker};

pub struct MySqlLocker;

impl Locker for MySqlLocker {
    type DB = ::sqlx::MySql;

    async fn with_locking<T, F>(
        pool: &sqlx::Pool<Self::DB>,
        key: &str,
        timeout: Option<std::time::Duration>,
        f: F,
    ) -> crate::Result<()>
    where
        F: AsyncFnOnce(&mut ::sqlx::Transaction<'static, Self::DB>) -> T,
    {
        let mut tx = pool.begin().await?;

        let timeout = timeout.unwrap_or_default().as_secs();
        let signal: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?,?)")
            .bind(key)
            .bind(timeout)
            .fetch_optional(&mut *tx)
            .await?;

        match signal {
            Some(1) => Ok(()),
            Some(0) => Err(Error::FailedToGetLock(key.to_string())),
            Some(signal) => Err(Error::MySqlUnknownSignal(signal)),
            None => Err(Error::MySqlReturnedNull),
        }?;

        f(&mut tx).await;

        sqlx::query("DO RELEASE_LOCK(?)")
            .bind(key)
            .fetch_optional(&mut *tx)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_matches};
    use std::time::Duration;
    use tokio::time::sleep;

    use super::*;

    use sqlx::MySqlPool;

    #[sqlx::test]
    async fn different_sessions_cannot_acquire_the_same_lock(pool: MySqlPool) -> sqlx::Result<()> {
        let (r1, r2) = tokio::join!(
            MySqlLocker::with_locking(
                &pool,
                "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
                Duration::from_secs(1).into(),
                async |_| {
                    sleep(Duration::from_secs(2)).await;
                },
            ),
            MySqlLocker::with_locking(
                &pool,
                "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
                Duration::from_secs(1).into(),
                async |_| {
                    sleep(Duration::from_secs(2)).await;
                },
            )
        );

        match (&r1, &r2) {
            (Ok(()), Err(_)) | (Err(_), Ok(())) => {
                ()
            }
            other => panic!("expected one Ok and one FailedToGetLock, got: {:?}", other),
        }

        let r = MySqlLocker::with_locking(
            &pool,
            "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
            Duration::from_secs(1).into(),
            async |_| {
            },
        )
        .await;

        assert_matches!(r, Ok(()));

        Ok(())
    }

    #[sqlx::test]
    async fn second_waits_then_acquires(pool: MySqlPool) -> sqlx::Result<()> {
        let (r1, r2) = tokio::join!(
            MySqlLocker::with_locking(
                &pool,
                "Cvw8utptkckId0IVIUDj612G00sjJ7O42FeMEfL07VQLYfH3nAq0eYKf60g082ui",
                Duration::from_secs(2).into(),
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            ),
            MySqlLocker::with_locking(
                &pool,
                "Cvw8utptkckId0IVIUDj612G00sjJ7O42FeMEfL07VQLYfH3nAq0eYKf60g082ui",
                Duration::from_secs(2).into(),
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            )
        );

        assert_matches!(r1, Ok(()));
        assert_matches!(r2, Ok(()));

        Ok(())
    }

    #[sqlx::test]
    async fn no_wait(pool: MySqlPool) -> sqlx::Result<()> {
        let (r1, r2) = tokio::join!(
            MySqlLocker::with_locking(
                &pool,
                "LjoiSBmBcdKIng3aBIsf0Yqi8oeTKH1UkRQHfKlFe5fBsDYjhRDEwOwtSUr8ewG3",
                None,
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            ),
            MySqlLocker::with_locking(
                &pool,
                "LjoiSBmBcdKIng3aBIsf0Yqi8oeTKH1UkRQHfKlFe5fBsDYjhRDEwOwtSUr8ewG3",
                None,
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            )
        );

        match (&r1, &r2) {
            (Ok(()), Err(_)) | (Err(_), Ok(())) => {
                ()
            }
            other => panic!("expected one Ok and one FailedToGetLock, got: {:?}", other),
        }

        let r = MySqlLocker::with_locking(
            &pool,
            "LjoiSBmBcdKIng3aBIsf0Yqi8oeTKH1UkRQHfKlFe5fBsDYjhRDEwOwtSUr8ewG3",
            Duration::from_secs(1).into(),
            async |_| {
            },
        )
        .await;

        assert_matches!(r, Ok(()));

        Ok(())
    }

    #[sqlx::test]
    async fn locck_with_empty_text(pool: MySqlPool) -> sqlx::Result<()> {
        let r = MySqlLocker::with_locking(&pool, "", Duration::from_secs(1).into(), async |_| {
            sleep(Duration::from_secs(1)).await;
        })
        .await;

        assert_matches!(r, Err(_));

        Ok(())
    }

    #[sqlx::test]
    async fn lock_with_text_longer_than_64(pool: MySqlPool) -> sqlx::Result<()> {
        let r = MySqlLocker::with_locking(
            &pool,
            "G2l1litxGfagbBWcQUymJ7cqYVyqQFPsr4JoimK4eXMRdN5n8tcofOYUJhEMHcbVH",
            Duration::from_secs(1).into(),
            async |_| {
                sleep(Duration::from_secs(1)).await;
            },
        )
        .await;

        assert_matches!(r, Ok(()));

        Ok(())
    }
}
