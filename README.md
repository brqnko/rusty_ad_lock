# rusty_ad_lock

[![CI](https://github.com/brqnko/rusty_ad_lock/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/brqnko/rusty_ad_lock/actions)

advisory lock implementation for mysql, postgres, and std collection, using sqlx

## Installation

```toml
rusty_ad_lock = { git = "https://github.com/brqnko/rusty_ad_lock.git" }
```

## How to use

```rs
// MySQL
let r = MySqlLocker::with_locking(
    &pool, // connection pool
    "key", // key to get locked
    Duration::from_secs(1).into(), // timeout duration. if it can't get lock in 1 sec, with_locking will return Err
    async |_| { // closure that executed while the key is locked
        sleep(Duration::from_secs(1)).await;
    },
)
.await;

// PostgreSQL
let r = PostgresLocker::with_locking(
    &pool, // connection pool
    "key", // key to get locked
    Duration::from_secs(1).into(), // timeout duration. if it can't get lock in 1 sec, with_locking will return Err
    async |_| { // closure that executed while the key is locked
        sleep(Duration::from_secs(1)).await;
    },
)
.await;

// collection(uses Mutex, HashMap)
let r = StdCollectionLocker::with_locking(
    &pool, // connection pool
    "key", // key to get locked
    Duration::from_secs(1).into(), // timeout duration. if it can't get lock in 1 sec, with_locking will return Err
    async |_| { // closure that executed while the key is locked
        sleep(Duration::from_secs(1)).await;
    },
)
.await;
```

## Contribution

pull requests and issues are welcome

## License

Apache 2.0
