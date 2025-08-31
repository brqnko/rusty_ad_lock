use std::marker::PhantomData;

use crate::Locker;

pub struct StdCollectionLocker<D: sqlx::Database> {
    _marker: PhantomData<D>,
}

impl<D: sqlx::Database> Locker for StdCollectionLocker<D> {
    type DB = D;

    async fn with_locking<T, F>(
        pool: &sqlx::Pool<Self::DB>,
        key: &str,
        timeout: Option<std::time::Duration>,
        f: F,
    ) -> super::Result<()>
    where
        F: AsyncFnOnce(&mut sqlx::Transaction<'static, Self::DB>) -> T,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_matches;
    use std::time::Duration;
    use tokio::time::sleep;

    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn different_sessions_cannot_acquire_the_same_lock(pool: SqlitePool) -> sqlx::Result<()> {
        let (r1, r2) = tokio::join!(
            StdCollectionLocker::with_locking(
                &pool,
                "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
                Duration::from_secs(1).into(),
                async |_| {
                    sleep(Duration::from_secs(2)).await;
                },
            ),
            StdCollectionLocker::with_locking(
                &pool,
                "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
                Duration::from_secs(1).into(),
                async |_| {
                    sleep(Duration::from_secs(2)).await;
                },
            )
        );

        match (&r1, &r2) {
            (Ok(()), Err(_)) | (Err(_), Ok(())) => (),
            other => panic!("expected one Ok and one FailedToGetLock, got: {:?}", other),
        }

        let r = StdCollectionLocker::with_locking(
            &pool,
            "ivcK1ms0G8xoI5aA40BMkiI2aVlhyM025EGFv1nJxNIC50pJovn2Vn1i7IKlnqYB",
            Duration::from_secs(1).into(),
            async |_| {},
        )
        .await;

        assert_matches!(r, Ok(()));

        Ok(())
    }

    #[sqlx::test]
    async fn second_waits_then_acquires(pool: SqlitePool) -> sqlx::Result<()> {
        let (r1, r2) = tokio::join!(
            StdCollectionLocker::with_locking(
                &pool,
                "Cvw8utptkckId0IVIUDj612G00sjJ7O42FeMEfL07VQLYfH3nAq0eYKf60g082ui",
                Duration::from_secs(2).into(),
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            ),
            StdCollectionLocker::with_locking(
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
    async fn no_wait(pool: SqlitePool) -> sqlx::Result<()> {
        let (r1, r2) = tokio::join!(
            StdCollectionLocker::with_locking(
                &pool,
                "LjoiSBmBcdKIng3aBIsf0Yqi8oeTKH1UkRQHfKlFe5fBsDYjhRDEwOwtSUr8ewG3",
                None,
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            ),
            StdCollectionLocker::with_locking(
                &pool,
                "LjoiSBmBcdKIng3aBIsf0Yqi8oeTKH1UkRQHfKlFe5fBsDYjhRDEwOwtSUr8ewG3",
                None,
                async |_| {
                    sleep(Duration::from_secs(1)).await;
                },
            )
        );

        match (&r1, &r2) {
            (Ok(()), Err(_)) | (Err(_), Ok(())) => (),
            other => panic!("expected one Ok and one FailedToGetLock, got: {:?}", other),
        }

        let r = StdCollectionLocker::with_locking(
            &pool,
            "LjoiSBmBcdKIng3aBIsf0Yqi8oeTKH1UkRQHfKlFe5fBsDYjhRDEwOwtSUr8ewG3",
            Duration::from_secs(1).into(),
            async |_| {},
        )
        .await;

        assert_matches!(r, Ok(()));

        Ok(())
    }

    #[sqlx::test]
    async fn locck_with_empty_text(pool: SqlitePool) -> sqlx::Result<()> {
        let r = StdCollectionLocker::with_locking(
            &pool,
            "",
            Duration::from_secs(1).into(),
            async |_| {
                sleep(Duration::from_secs(1)).await;
            },
        )
        .await;

        assert_matches!(r, Ok(_));

        Ok(())
    }

    #[sqlx::test]
    async fn lock_with_text_longer_than_64(pool: SqlitePool) -> sqlx::Result<()> {
        let r = StdCollectionLocker::with_locking(
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
