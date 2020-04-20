use sqlx::{database::Database, Connect};

fn setup_if_needed() {
    let _ = dotenv::dotenv();
    let _ = env_logger::builder().is_test(true).try_init();
}

// Make a new connection
// Ensure [dotenv] and [env_logger] have been setup
pub async fn new<DB>() -> anyhow::Result<DB::Connection>
where
    DB: Database,
{
    setup_if_needed();

    Ok(DB::Connection::connect(&dotenv::var("DATABASE_URL")?).await?)
}

// Test type encoding and decoding
#[macro_export]
macro_rules! test_type {
    ($name:ident<$ty:ty>($db:ident, $sql:literal, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::test_prepared_type!($name($db, $ty, $sql, $($text == $value),+));
        $crate::test_unprepared_type!($name($db, $ty, $($text == $value),+));
    };

    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            $crate::test_prepared_type!($name($db, $ty, $crate::[< $db _query_for_test_prepared_type >]!(), $($text == $value),+));
            $crate::test_unprepared_type!($name($db, $ty, $($text == $value),+));
        }
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::test_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type decoding for the simple (unprepared) query API
#[macro_export]
macro_rules! test_unprepared_type {
    ($name:ident($db:ident, $ty:ty, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            #[sqlx_rt::test]
            async fn [< test_unprepared_type_ $name >] () -> anyhow::Result<()> {
                use sqlx::prelude::*;
                use futures::TryStreamExt;

                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!("SELECT {}", $text);
                    let mut s = conn.fetch(&*query);
                    let row = s.try_next().await?.unwrap();
                    let rec = row.try_get::<$ty, _>(0)?;

                    assert!($value == rec);

                    drop(s);
                )+

                Ok(())
            }
        }
    }
}

// Test type encoding and decoding for the prepared query API
#[macro_export]
macro_rules! test_prepared_type {
    ($name:ident($db:ident, $ty:ty, $sql:expr, $($text:literal == $value:expr),+ $(,)?)) => {
        paste::item! {
            #[sqlx_rt::test]
            async fn [< test_prepared_type_ $name >] () -> anyhow::Result<()> {
                let mut conn = sqlx_test::new::<$db>().await?;

                $(
                    let query = format!($sql, $text);

                    println!("about to exec: {}", query);

                    let rec: (bool, $ty, $ty) = sqlx::query_as(&query)
                        .bind($value)
                        .bind($value)
                        .fetch_one(&mut conn)
                        .await?;

                    assert!(rec.0,
                            "[1] DB value mismatch; given value: {:?}\n\
                             as returned: {:?}\n\
                             round-trip: {:?}",
                            $value, rec.1, rec.2);

                    assert_eq!($value, rec.1,
                            "[2] DB value mismatch; given value: {:?}\n\
                                     as returned: {:?}\n\
                                     round-trip: {:?}",
                                    $value, rec.1, rec.2);

                    assert_eq!($value, rec.2,
                            "[3] DB value mismatch; given value: {:?}\n\
                                     as returned: {:?}\n\
                                     round-trip: {:?}",
                                    $value, rec.1, rec.2);
                )+

                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! MySql_query_for_test_prepared_type {
    () => {
        "SELECT {0} <=> ?, {0}, ?"
    };
}

#[macro_export]
macro_rules! Sqlite_query_for_test_prepared_type {
    () => {
        "SELECT {0} is ?, {0}, ?"
    };
}

#[macro_export]
macro_rules! Postgres_query_for_test_prepared_type {
    () => {
        "SELECT {0} is not distinct from $1, {0}, $2"
    };
}
