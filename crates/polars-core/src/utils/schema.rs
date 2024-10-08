use polars_utils::format_pl_smallstr;

use crate::prelude::*;

/// Convert a collection of [`DataType`] into a schema.
///
/// Field names are set as `column_0`, `column_1`, and so on.
///
/// [`DataType`]: crate::datatypes::DataType
pub fn dtypes_to_schema<I>(dtypes: I) -> Schema
where
    I: IntoIterator<Item = DataType>,
{
    dtypes
        .into_iter()
        .enumerate()
        .map(|(i, dtype)| Field::new(format_pl_smallstr!("column_{i}"), dtype))
        .collect()
}
