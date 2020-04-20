use futures_util::{stream, StreamExt, TryStreamExt};
use hashbrown::HashMap;

use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::postgres::message::{ParameterDescription, RowDescription};
use crate::postgres::statement::Column;
use crate::postgres::{PgConnection, PgTypeInfo};
use crate::query_as::query_as;

impl PgConnection {
    pub(super) async fn handle_row_description(
        &mut self,
        desc: RowDescription,
        should_fetch: bool,
    ) -> Result<(Vec<Column>, HashMap<UStr, usize>), Error> {
        let mut columns = Vec::with_capacity(desc.fields.len());
        let mut names = HashMap::with_capacity(desc.fields.len());

        for (index, field) in desc.fields.into_iter().enumerate() {
            let name = UStr::from(field.name);

            let type_info = self
                .fetch_type_info_by_id(field.data_type_id, should_fetch)
                .await?;

            let column = Column {
                name: name.clone(),
                type_info,
                relation_id: field.relation_id,
                relation_attribute_no: field.relation_attribute_no,
            };

            columns.push(column);
            names.insert(name, index);
        }

        Ok((columns, names))
    }

    pub(super) async fn handle_parameter_description(
        &mut self,
        desc: ParameterDescription,
    ) -> Result<Vec<PgTypeInfo>, Error> {
        let mut params = Vec::with_capacity(desc.types.len());

        for ty in desc.types {
            params.push(self.fetch_type_info_by_id(ty, true).await?);
        }

        Ok(params)
    }

    async fn fetch_type_info_by_id(
        &mut self,
        id: u32,
        should_fetch: bool,
    ) -> Result<PgTypeInfo, Error> {
        // first we check if this is a built-in type
        // in the average application, the vast majority of checks should flow through this
        if let Some(info) = PgTypeInfo::try_from_id(id) {
            return Ok(info);
        }

        // next we check a local cache for user-defined type names <-> object id
        if let Some(name) = self.cache_type_name.get(&id) {
            return Ok(PgTypeInfo {
                id: Some(id),
                name: name.clone(),
            });
        }

        // fallback to asking the database directly for a type name
        let name = if should_fetch {
            let (name,): (String,) =
                query_as("SELECT typname FROM pg_catalog.pg_type WHERE oid = $1")
                    .bind(id)
                    .fetch_one(&mut *self)
                    .await?;

            let name = UStr::from(name);

            // cache the type name <-> oid relationship in a paired hashmap
            // so we don't come down this road again
            self.cache_type_id.insert(name.clone(), id);
            self.cache_type_name.insert(id, name.clone());

            name
        } else {
            // we are not in a place that *can* run a query
            // this generally means we are in the middle of another query
            // this _should_ only happen for complex types sent through the TEXT protocol
            // we're open to ideas to correct this.. but it'd probably be more efficient to figure
            // out a way to "prime" the type cache for connections rather than make this
            // fallback work correctly for complex user-defined types for the TEXT protocol
            UStr::Static("")
        };

        Ok(PgTypeInfo { id: Some(id), name })
    }
}
