//! PostgreSQL database driver and related types.

mod arguments;
mod connection;
mod database;
mod error;
mod io;
mod message;
mod options;
mod row;
mod statement;
mod type_info;
pub mod types;
mod value;

pub use arguments::PgArguments;
pub use connection::PgConnection;
pub use database::Postgres;
pub use error::{PgDatabaseError, PgErrorPosition};
pub use message::PgSeverity;
pub use options::{PgConnectOptions, PgSslMode};
pub use row::PgRow;
pub use type_info::PgTypeInfo;
pub use value::{PgValue, PgValueFormat, PgValueRef};

// /// An alias for [`Pool`][crate::pool::Pool], specialized for Postgres.
// pub type PgPool = crate::pool::Pool<PgConnection>;
