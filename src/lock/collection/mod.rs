use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::{Arc, LazyLock, Mutex},
};

use sqlx::ConnectOptions;
use tokio::sync::broadcast;

use crate::{Error, Locker};

pub struct StdCollectionLocker<D: sqlx::Database> {
    _marker: PhantomData<D>,
}

#[derive(Clone, Debug)]
enum Event {
    Released { url: Arc<String>, key: Arc<String> },
}

const CHANNEL_BUFFER_SIZE: usize = 32;

static STATE: LazyLock<Mutex<HashMap<Arc<String>, HashSet<Arc<String>>>>> =
    LazyLock::new(|| Mutex::default());

static BROADCAST: LazyLock<broadcast::Sender<Event>> = LazyLock::new(|| {
    let (sx, _rx) = broadcast::channel(CHANNEL_BUFFER_SIZE);
    sx
});

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
        let url = Arc::new(pool.connect_options().to_url_lossy().to_string());
        let key = Arc::new(key.to_owned());

        let mut tx = pool.begin().await?;

        fn try_lock(url: &Arc<String>, key: &Arc<String>) -> bool {
            let mut state = STATE.lock().unwrap();
            state
                .entry(Arc::clone(url))
                .or_default()
                .insert(Arc::clone(key))
        }

        // まず即時取得を試みる
        if !try_lock(&url, &key) {
            // 待たない設定なら即失敗
            let Some(dur) = timeout else {
                return Err(Error::FailedToGetLock((*key).to_string()));
            };

            // 指定期間待って、目的の (url, key) が解放されたら再取得を試みる
            let mut rx = BROADCAST.subscribe();
            let acquired = tokio::time::timeout(dur, async {
                loop {
                    match rx.recv().await {
                        Ok(Event::Released { url: u, key: k })
                            if u.eq(&url) && k.eq(&key) && try_lock(&url, &key) =>
                        {
                            break true;
                        }
                        Ok(_) => { /* 別のロック解放: 無視 */ }
                        Err(_) => break false, // チャネルが閉じた等
                    }
                }
            })
            .await
            .ok()
            .unwrap_or(false);

            if !acquired {
                return Err(Error::FailedToGetLock((*key).to_string()));
            }
        }

        f(&mut tx).await;

        // ロックを解除する
        let mut state = STATE.lock().unwrap();
        state.get_mut(&url).map(|set| set.remove(&key));
        drop(state);
        // ロックを開放したことを送信
        // NOTE: エラーが来ても、それは受診者が0なことを表しているだけ
        let _ = BROADCAST.send(Event::Released {
            url: Arc::clone(&url),
            key,
        });

        Ok(())
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
