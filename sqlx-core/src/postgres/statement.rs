use hashbrown::HashMap;

use crate::ext::ustr::UStr;
use crate::postgres::PgTypeInfo;

// A prepared statement
#[derive(Debug)]
pub(crate) struct Statement {
    pub(crate) id: u32,
    pub(crate) param_types: Option<Vec<PgTypeInfo>>,
    pub(crate) columns: Vec<Column>,
    pub(crate) column_names: HashMap<UStr, usize>,
}

impl Statement {
    pub(crate) fn empty() -> Self {
        Self {
            id: 0,
            param_types: None,
            columns: Vec::new(),
            column_names: HashMap::new(),
        }
    }
}

// Result column of a prepared statement
// See RowDescription/Field for more information
#[derive(Debug)]
pub(crate) struct Column {
    pub(crate) name: UStr,
    pub(crate) type_info: PgTypeInfo,
    pub(crate) relation_id: Option<u32>,
    pub(crate) relation_attribute_no: Option<u16>,
}
