use polars_core::config;
use polars_core::prelude::*;
use polars_parquet::read::statistics::{deserialize, Statistics};
use polars_parquet::read::RowGroupMetadata;

use crate::predicates::{BatchStats, ColumnStats, ScanIOPredicate};

/// Collect the statistics in a row-group
pub fn collect_statistics_with_live_columns(
    md: &RowGroupMetadata,
    schema: &ArrowSchema,
    pl_schema: &SchemaRef,
    live_columns: &PlIndexSet<PlSmallStr>,
) -> PolarsResult<Option<BatchStats>> {
    // TODO! fix this performance. This is a full sequential scan.
    let stats = live_columns
        .iter()
        .map(|c| {
            let field = schema.get(c).unwrap();

            let default_fn = || ColumnStats::new(field.into(), None, None, None);

            // This can be None in the allow_missing_columns case.
            let Some(mut iter) = md.columns_under_root_iter(&field.name) else {
                return Ok(default_fn());
            };

            let statistics = deserialize(field, &mut iter)?;
            assert!(iter.next().is_none());

            // We don't support reading nested statistics for now. It does not really make any
            // sense at the moment with how we structure statistics.
            let Some(Statistics::Column(stats)) = statistics else {
                return Ok(default_fn());
            };

            let stats = stats.into_arrow()?;

            let null_count = stats
                .null_count
                .map(|x| Scalar::from(x).into_series(PlSmallStr::EMPTY));
            let min_value = stats
                .min_value
                .map(|x| Series::try_from((PlSmallStr::EMPTY, x)).unwrap());
            let max_value = stats
                .max_value
                .map(|x| Series::try_from((PlSmallStr::EMPTY, x)).unwrap());

            Ok(ColumnStats::new(
                field.into(),
                null_count,
                min_value,
                max_value,
            ))
        })
        .collect::<PolarsResult<Vec<_>>>()?;

    if stats.is_empty() {
        return Ok(None);
    }

    Ok(Some(BatchStats::new(
        pl_schema.clone(),
        stats,
        Some(md.num_rows()),
    )))
}

/// Collect the statistics in a row-group
pub fn collect_statistics(
    md: &RowGroupMetadata,
    schema: &ArrowSchema,
) -> PolarsResult<Option<BatchStats>> {
    // TODO! fix this performance. This is a full sequential scan.
    let stats = schema
        .iter_values()
        .map(|field| {
            let default_fn = || ColumnStats::new(field.into(), None, None, None);

            // This can be None in the allow_missing_columns case.
            let Some(mut iter) = md.columns_under_root_iter(&field.name) else {
                return Ok(default_fn());
            };

            let statistics = deserialize(field, &mut iter)?;
            assert!(iter.next().is_none());

            // We don't support reading nested statistics for now. It does not really make any
            // sense at the moment with how we structure statistics.
            let Some(Statistics::Column(stats)) = statistics else {
                return Ok(default_fn());
            };

            let stats = stats.into_arrow()?;

            let null_count = stats
                .null_count
                .map(|x| Scalar::from(x).into_series(PlSmallStr::EMPTY));
            let min_value = stats
                .min_value
                .map(|x| Series::try_from((PlSmallStr::EMPTY, x)).unwrap());
            let max_value = stats
                .max_value
                .map(|x| Series::try_from((PlSmallStr::EMPTY, x)).unwrap());

            Ok(ColumnStats::new(
                field.into(),
                null_count,
                min_value,
                max_value,
            ))
        })
        .collect::<PolarsResult<Vec<_>>>()?;

    if stats.is_empty() {
        return Ok(None);
    }

    Ok(Some(BatchStats::new(
        Arc::new(Schema::from_arrow_schema(schema)),
        stats,
        Some(md.num_rows()),
    )))
}

pub fn read_this_row_group(
    predicate: Option<&ScanIOPredicate>,
    md: &RowGroupMetadata,
    schema: &ArrowSchema,
) -> PolarsResult<bool> {
    if std::env::var("POLARS_NO_PARQUET_STATISTICS").is_ok() {
        return Ok(true);
    }

    let mut should_read = true;

    if let Some(predicate) = predicate {
        if let Some(pred) = &predicate.skip_batch_predicate {
            if let Some(stats) = collect_statistics(md, schema)? {
                let stats = PlIndexMap::from_iter(stats.column_stats().iter().map(|col| {
                    (
                        col.field_name().clone(),
                        crate::predicates::ColumnStatistics {
                            dtype: stats.schema().get(col.field_name()).unwrap().clone(),
                            min: col
                                .to_min()
                                .map_or(AnyValue::Null, |s| s.get(0).unwrap().into_static()),
                            max: col
                                .to_max()
                                .map_or(AnyValue::Null, |s| s.get(0).unwrap().into_static()),
                            null_count: col.null_count().map(|nc| nc as IdxSize),
                        },
                    )
                }));
                let pred_result = pred.can_skip_batch(
                    md.num_rows() as IdxSize,
                    predicate.live_columns.as_ref(),
                    stats,
                );

                // a parquet file may not have statistics of all columns
                match pred_result {
                    Err(PolarsError::ColumnNotFound(errstr)) => {
                        return Err(PolarsError::ColumnNotFound(errstr))
                    },
                    Ok(true) => should_read = false,
                    _ => {},
                }
            }
        } else if let Some(pred) = predicate.predicate.as_stats_evaluator() {
            if let Some(stats) = collect_statistics(md, schema)? {
                let pred_result = pred.should_read(&stats);

                // a parquet file may not have statistics of all columns
                match pred_result {
                    Err(PolarsError::ColumnNotFound(errstr)) => {
                        return Err(PolarsError::ColumnNotFound(errstr))
                    },
                    Ok(false) => should_read = false,
                    _ => {},
                }
            }
        }

        if config::verbose() {
            if should_read {
                eprintln!(
                    "parquet row group must be read, statistics not sufficient for predicate."
                );
            } else {
                eprintln!("parquet row group can be skipped, the statistics were sufficient to apply the predicate.");
            }
        }
    }

    Ok(should_read)
}
