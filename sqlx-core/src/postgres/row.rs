use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use crate::database::HasValueRef;
use crate::error::Error;
use crate::postgres::message::DataRow;
use crate::postgres::statement::Statement;
use crate::postgres::value::PgValueFormat;
use crate::postgres::{PgValueRef, Postgres};
use crate::row::{ColumnIndex, Row};

/// Implementation of [`Row`] for PostgreSQL.
pub struct PgRow {
    pub(crate) data: DataRow,
    pub(crate) format: PgValueFormat,
    pub(crate) statement: Arc<Statement>,
}

impl crate::row::private_row::Sealed for PgRow {}

impl Row for PgRow {
    type Database = Postgres;

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<<Self::Database as HasValueRef>::ValueRef, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.statement.columns[index];
        let value = self.data.get(index);

        Ok(PgValueRef {
            format: self.format,
            row: Some(&self.data.storage),
            type_info: column.type_info.clone(),
            value,
        })
    }
}

impl ColumnIndex<PgRow> for &'_ str {
    fn index(&self, row: &PgRow) -> Result<usize, Error> {
        row.statement
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
