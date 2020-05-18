use futures::TryStreamExt;
use sqlx::postgres::PgRow;
use sqlx::{postgres::Postgres, Connection, Executor, Row};
use sqlx_test::new;
use std::time::Duration;

#[sqlx_rt::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let value = sqlx::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[sqlx_rt::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let cnt = sqlx::query("INSERT INTO users (id) VALUES ($1)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(cnt, 1);
    }

    let sum: i32 = sqlx::query("SELECT id FROM users")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

// https://github.com/launchbadge/sqlx/issues/104
#[sqlx_rt::test]
async fn it_can_return_interleaved_nulls_issue_104() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let tuple = sqlx::query("SELECT NULL, 10::INT, NULL, 20::INT, NULL, 40::INT, NULL, 80::INT")
        .try_map(|row: PgRow| {
            Ok((
                row.get::<Option<i32>, _>(0),
                row.get::<Option<i32>, _>(1),
                row.get::<Option<i32>, _>(2),
                row.get::<Option<i32>, _>(3),
                row.get::<Option<i32>, _>(4),
                row.get::<Option<i32>, _>(5),
                row.get::<Option<i32>, _>(6),
                row.get::<Option<i32>, _>(7),
            ))
        })
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(tuple.0, None);
    assert_eq!(tuple.1, Some(10));
    assert_eq!(tuple.2, None);
    assert_eq!(tuple.3, Some(20));
    assert_eq!(tuple.4, None);
    assert_eq!(tuple.5, Some(40));
    assert_eq!(tuple.6, None);
    assert_eq!(tuple.7, Some(80));

    Ok(())
}

#[sqlx_rt::test]
async fn it_can_query_scalar() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let scalar: i32 = sqlx::query_scalar("SELECT 42").fetch_one(&mut conn).await?;
    assert_eq!(scalar, 42);

    let scalar: Option<i32> = sqlx::query_scalar("SELECT 42").fetch_one(&mut conn).await?;
    assert_eq!(scalar, Some(42));

    let scalar: Option<i32> = sqlx::query_scalar("SELECT NULL")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(scalar, None);

    let scalar: Option<i64> = sqlx::query_scalar("SELECT 42::bigint")
        .fetch_optional(&mut conn)
        .await?;
    assert_eq!(scalar, Some(42));

    let scalar: Option<i16> = sqlx::query_scalar("").fetch_optional(&mut conn).await?;
    assert_eq!(scalar, None);

    Ok(())
}

#[sqlx_rt::test]
/// This is seperate from `it_can_query_scalar` because while implementing it I ran into a
/// bug which that prevented `Vec<i32>` from compiling but allowed Vec<Option<i32>>.
async fn it_can_query_all_scalar() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let scalar: Vec<i32> = sqlx::query_scalar("SELECT $1")
        .bind(42)
        .fetch_all(&mut conn)
        .await?;
    assert_eq!(scalar, vec![42]);

    let scalar: Vec<Option<i32>> = sqlx::query_scalar("SELECT $1 UNION ALL SELECT NULL")
        .bind(42)
        .fetch_all(&mut conn)
        .await?;
    assert_eq!(scalar, vec![Some(42), None]);

    Ok(())
}

// #[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-tokio", tokio::test)]
// async fn it_can_work_with_transactions() -> anyhow::Result<()> {
//     let mut conn = new::<Postgres>().await?;
//
//     conn.execute("CREATE TABLE IF NOT EXISTS _sqlx_users_1922 (id INTEGER PRIMARY KEY)")
//         .await?;
//
//     conn.execute("TRUNCATE _sqlx_users_1922").await?;
//
//     // begin .. rollback
//
//     let mut tx = conn.begin().await?;
//
//     sqlx::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
//         .bind(10_i32)
//         .execute(&mut tx)
//         .await?;
//
//     conn = tx.rollback().await?;
//
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
//         .fetch_one(&mut conn)
//         .await?;
//
//     assert_eq!(count, 0);
//
//     // begin .. commit
//
//     let mut tx = conn.begin().await?;
//
//     sqlx::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
//         .bind(10_i32)
//         .execute(&mut tx)
//         .await?;
//
//     conn = tx.commit().await?;
//
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
//         .fetch_one(&mut conn)
//         .await?;
//
//     assert_eq!(count, 1);
//
//     // begin .. (drop)
//
//     {
//         let mut tx = conn.begin().await?;
//
//         sqlx::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
//             .bind(20_i32)
//             .execute(&mut tx)
//             .await?;
//     }
//
//     conn = new::<Postgres>().await?;
//
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
//         .fetch_one(&mut conn)
//         .await?;
//
//     assert_eq!(count, 1);
//
//     Ok(())
// }
//
// #[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-tokio", tokio::test)]
// async fn it_can_work_with_nested_transactions() -> anyhow::Result<()> {
//     let mut conn = new::<Postgres>().await?;
//
//     conn.execute("CREATE TABLE IF NOT EXISTS _sqlx_users_2523 (id INTEGER PRIMARY KEY)")
//         .await?;
//
//     conn.execute("TRUNCATE _sqlx_users_2523").await?;
//
//     // begin
//     let mut tx = conn.begin().await?;
//
//     // insert a user
//     sqlx::query("INSERT INTO _sqlx_users_2523 (id) VALUES ($1)")
//         .bind(50_i32)
//         .execute(&mut tx)
//         .await?;
//
//     // begin once more
//     let mut tx = tx.begin().await?;
//
//     // insert another user
//     sqlx::query("INSERT INTO _sqlx_users_2523 (id) VALUES ($1)")
//         .bind(10_i32)
//         .execute(&mut tx)
//         .await?;
//
//     // never mind, rollback
//     let mut tx = tx.rollback().await?;
//
//     // did we really?
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_users_2523")
//         .fetch_one(&mut tx)
//         .await?;
//
//     assert_eq!(count, 1);
//
//     // actually, commit
//     let mut conn = tx.commit().await?;
//
//     // did we really?
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_users_2523")
//         .fetch_one(&mut conn)
//         .await?;
//
//     assert_eq!(count, 1);
//
//     Ok(())
// }
//
// #[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-tokio", tokio::test)]
// async fn it_can_rollback_nested_transactions() -> anyhow::Result<()> {
//     let mut conn = new::<Postgres>().await?;
//
//     conn.execute("CREATE TABLE IF NOT EXISTS _sqlx_users_512412 (id INTEGER PRIMARY KEY)")
//         .await?;
//
//     conn.execute("TRUNCATE _sqlx_users_512412").await?;
//
//     // begin
//     let mut tx = conn.begin().await?;
//
//     // insert a user
//     sqlx::query("INSERT INTO _sqlx_users_512412 (id) VALUES ($1)")
//         .bind(50_i32)
//         .execute(&mut tx)
//         .await?;
//
//     // begin once more
//     let mut tx = tx.begin().await?;
//
//     // insert another user
//     sqlx::query("INSERT INTO _sqlx_users_512412 (id) VALUES ($1)")
//         .bind(10_i32)
//         .execute(&mut tx)
//         .await?;
//
//     // stop the phone, drop the entire transaction
//     tx.close().await?;
//
//     // did we really?
//     let mut conn = new::<Postgres>().await?;
//     let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_users_512412")
//         .fetch_one(&mut conn)
//         .await?;
//
//     assert_eq!(count, 0);
//
//     Ok(())
// }
//
// // run with `cargo test --features postgres -- --ignored --nocapture pool_smoke_test`
// #[ignore]
// #[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-tokio", tokio::test)]
// async fn pool_smoke_test() -> anyhow::Result<()> {
//     #[cfg(feature = "runtime-tokio")]
//     use tokio::{task::spawn, time::delay_for as sleep, time::timeout};
//
//     #[cfg(feature = "runtime-async-std")]
//     use async_std::{future::timeout, task::sleep, task::spawn};
//
//     eprintln!("starting pool");
//
//     let pool = PgPool::builder()
//         .connect_timeout(Duration::from_secs(5))
//         .min_size(5)
//         .max_size(10)
//         .build(&dotenv::var("DATABASE_URL")?)
//         .await?;
//
//     // spin up more tasks than connections available, and ensure we don't deadlock
//     for i in 0..20 {
//         let pool = pool.clone();
//         spawn(async move {
//             loop {
//                 if let Err(e) = sqlx::query("select 1 + 1").execute(&pool).await {
//                     eprintln!("pool task {} dying due to {}", i, e);
//                     break;
//                 }
//             }
//         });
//     }
//
//     for _ in 0..5 {
//         let pool = pool.clone();
//         spawn(async move {
//             while !pool.is_closed() {
//                 // drop acquire() futures in a hot loop
//                 // https://github.com/launchbadge/sqlx/issues/83
//                 drop(pool.acquire());
//             }
//         });
//     }
//
//     eprintln!("sleeping for 30 seconds");
//
//     sleep(Duration::from_secs(30)).await;
//
//     assert_eq!(pool.size(), 10);
//
//     eprintln!("closing pool");
//
//     timeout(Duration::from_secs(30), pool.close()).await?;
//
//     eprintln!("pool closed successfully");
//
//     Ok(())
// }

#[sqlx_rt::test]
async fn test_invalid_query() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute("definitely not a correct query")
        .await
        .unwrap_err();

    let mut s = conn.fetch("select 1");
    let row = s.try_next().await?.unwrap();

    assert_eq!(row.get::<i32, _>(0), 1i32);

    Ok(())
}

// #[cfg_attr(feature = "runtime-async-std", async_std::test)]
// #[cfg_attr(feature = "runtime-tokio", tokio::test)]
// async fn test_describe() -> anyhow::Result<()> {
//     let mut conn = new::<Postgres>().await?;
//
//     let _ = conn
//         .execute(
//             r#"
//         CREATE TEMP TABLE describe_test (
//             id SERIAL primary key,
//             name text not null,
//             hash bytea
//         )
//     "#,
//         )
//         .await?;
//
//     let describe = conn
//         .describe("select nt.*, false from describe_test nt")
//         .await?;
//
//     assert_eq!(describe.result_columns[0].non_null, Some(true));
//     assert_eq!(
//         describe.result_columns[0]
//             .type_info
//             .as_ref()
//             .unwrap()
//             .to_string(),
//         "INT4"
//     );
//     assert_eq!(describe.result_columns[1].non_null, Some(true));
//     assert_eq!(
//         describe.result_columns[1]
//             .type_info
//             .as_ref()
//             .unwrap()
//             .to_string(),
//         "TEXT"
//     );
//     assert_eq!(describe.result_columns[2].non_null, Some(false));
//     assert_eq!(
//         describe.result_columns[2]
//             .type_info
//             .as_ref()
//             .unwrap()
//             .to_string(),
//         "BYTEA"
//     );
//     assert_eq!(describe.result_columns[3].non_null, None);
//     assert_eq!(
//         describe.result_columns[3]
//             .type_info
//             .as_ref()
//             .unwrap()
//             .to_string(),
//         "BOOL"
//     );
//
//     Ok(())
// }
