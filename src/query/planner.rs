use crate::query::{Aggregation, Condition, Query};
use crate::schema::Table;
use crate::storage::StorageManager;
use crate::types::{DbError, Value};
use crate::DataType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

pub struct QueryEngine {
    storage: Arc<Mutex<StorageManager>>,
}

impl QueryEngine {
    pub fn new(storage: Arc<Mutex<StorageManager>>) -> Self {
        QueryEngine { storage }
    }

    pub fn execute(&mut self, query: Query) -> Result<Vec<Vec<Value>>, DbError> {
        match query {
            Query::Select {
                table,
                columns,
                condition,
            } => {
                let columns = if columns.is_empty() {
                    let storage_guard = self.storage.lock().unwrap();
                    storage_guard
                        .schema()
                        .get_table(&table)
                        .ok_or_else(|| DbError::InvalidData(format!("Table {} not found", table)))?
                        .columns
                        .iter()
                        .map(|c| c.name.clone())
                        .collect()
                } else {
                    columns
                };
                self.execute_select(&table, &columns, condition)
            }
            Query::SelectAggregate {
                table,
                aggregations,
                condition,
            } => self.execute_aggregate(&table, &aggregations, condition),
            Query::Join {
                left_table,
                right_table,
                left_column,
                right_column,
                columns,
                condition,
            } => self.execute_join(
                &left_table,
                &right_table,
                &left_column,
                &right_column,
                &columns,
                condition,
            ),
            Query::Insert { table, values } => {
                self.storage.lock().unwrap().insert_row(&table, values)?;
                Ok(vec![])
            }
            Query::CreateTable { table, columns } => {
                let table_def = Table {
                    name: table.clone(),
                    columns: columns
                        .into_iter()
                        .map(|(name, data_type)| crate::schema::Column { name, data_type })
                        .collect(),
                    row_count: 0,
                };
                self.storage.lock().unwrap().create_table(&table_def)?;
                Ok(vec![])
            }
            Query::Delete { table, condition } => {
                self.storage.lock().unwrap().delete_rows(&table, condition.as_ref())?;
                Ok(vec![])
            }
            Query::DropTable { table } => {
                self.storage.lock().unwrap().drop_table(&table)?;
                Ok(vec![])
            }
            Query::StartTransaction | Query::Commit | Query::Rollback => {
                Ok(vec![])
            }
        }
    }

    fn execute_select(
        &mut self,
        table: &str,
        columns: &[String],
        condition: Option<Condition>,
    ) -> Result<Vec<Vec<Value>>, DbError> {
        let table_def = {
            let storage_guard = self.storage.lock().unwrap();
            storage_guard
                .schema()
                .get_table(table)
                .ok_or_else(|| DbError::InvalidData(format!("Table {} not found", table)))?
                .clone()
        };

        for col in columns {
            if !table_def.columns.iter().any(|c| c.name == *col) {
                return Err(DbError::InvalidData(format!("Column {}.{} not found", table, col)));
            }
        }

        let mut required_columns = columns.to_vec();
        if let Some(ref cond) = condition {
            let condition_columns = crate::query::collect_condition_columns(cond);
            for col in condition_columns {
                if !table_def.columns.iter().any(|c| c.name == col) {
                    return Err(DbError::InvalidData(format!("Column {}.{} not found in condition", table, col)));
                }
                if !required_columns.contains(&col) {
                    required_columns.push(col);
                }
            }
        }

        let mut storage_guard = self.storage.lock().unwrap();
        let mut column_values = HashMap::new();
        let mut min_row_count = usize::MAX;
        for col in &required_columns {
            let values = storage_guard.read_column(table, col, condition.as_ref())?;
            min_row_count = min_row_count.min(values.len());
            column_values.insert(col.clone(), values);
        }

        // Parallelize row filtering and collection
        let result: Result<Vec<Vec<Value>>, DbError> = (0..min_row_count)
            .into_par_iter()
            .filter_map(|i| {
                if let Some(ref cond) = condition {
                    match crate::query::evaluator::evaluate_condition_row(cond, &column_values, i) {
                        Ok(true) => Some(Ok(columns
                            .iter()
                            .map(|col| column_values.get(col).unwrap()[i].clone())
                            .collect())),
                        Ok(false) => None,
                        Err(e) => Some(Err(e)),
                    }
                } else {
                    Some(Ok(columns
                        .iter()
                        .map(|col| column_values.get(col).unwrap()[i].clone())
                        .collect()))
                }
            })
            .collect();
        result
    }

    fn execute_aggregate(
        &mut self,
        table: &str,
        aggregations: &[Aggregation],
        condition: Option<Condition>,
    ) -> Result<Vec<Vec<Value>>, DbError> {
        let table_def = {
            let storage_guard = self.storage.lock().unwrap();
            storage_guard
                .schema()
                .get_table(table)
                .ok_or_else(|| DbError::InvalidData(format!("Table {} not found", table)))?
                .clone()
        };

        let mut storage_guard = self.storage.lock().unwrap();
        let mut results = Vec::new();
        for agg in aggregations {
            let column = match agg {
                Aggregation::Count => "ID".to_string(),
                Aggregation::Sum(col) | Aggregation::Avg(col) | Aggregation::Min(col) | Aggregation::Max(col) => col.clone(),
            };
            let col_def = table_def
                .get_column(&column)
                .ok_or_else(|| DbError::InvalidData(format!("Column {}.{} not found", table, column)))?;
            let values = storage_guard.read_column(table, &column, condition.as_ref())?;

            let result = match agg {
                Aggregation::Count => Value::Int32(values.len() as i32),
                Aggregation::Sum(_) => {
                    if col_def.data_type != DataType::Float32 && col_def.data_type != DataType::Int32 {
                        return Err(DbError::InvalidData(format!(
                            "SUM not supported for type {:?}", col_def.data_type
                        )));
                    }
                    values.iter().fold(Value::Float32(ordered_float::OrderedFloat(0.0)), |acc, v| {
                        match (acc.clone(), v) {
                            (Value::Float32(a), Value::Float32(b)) => Value::Float32(a + b),
                            (Value::Float32(a), Value::Int32(b)) => {
                                Value::Float32(a + ordered_float::OrderedFloat(*b as f32))
                            }
                            _ => acc,
                        }
                    })
                }
                Aggregation::Avg(_) => {
                    if col_def.data_type != DataType::Float32 && col_def.data_type != DataType::Int32 {
                        return Err(DbError::InvalidData(format!(
                            "AVG not supported for type {:?}", col_def.data_type
                        )));
                    }
                    let sum = values.iter().fold(Value::Float32(ordered_float::OrderedFloat(0.0)), |acc, v| {
                        match (acc.clone(), v) {
                            (Value::Float32(a), Value::Float32(b)) => Value::Float32(a + b),
                            (Value::Float32(a), Value::Int32(b)) => {
                                Value::Float32(a + ordered_float::OrderedFloat(*b as f32))
                            }
                            _ => acc,
                        }
                    });
                    match sum {
                        Value::Float32(s) if values.len() > 0 => {
                            Value::Float32(ordered_float::OrderedFloat(s.0 / values.len() as f32))
                        }
                        _ => Value::Float32(ordered_float::OrderedFloat(0.0)),
                    }
                }
                Aggregation::Min(_) => values
                    .iter()
                    .min_by(|a, b| a.cmp(b))
                    .cloned()
                    .unwrap_or(Value::Float32(ordered_float::OrderedFloat(0.0))),
                Aggregation::Max(_) => values
                    .iter()
                    .max_by(|a, b| a.cmp(b))
                    .cloned()
                    .unwrap_or(Value::Float32(ordered_float::OrderedFloat(0.0))),
            };
            results.push(result);
        }
        Ok(vec![results])
    }

    fn execute_join(
        &mut self,
        left_table: &str,
        right_table: &str,
        left_column: &str,
        right_column: &str,
        columns: &[String],
        condition: Option<Condition>,
    ) -> Result<Vec<Vec<Value>>, DbError> {
        let mut storage_guard = self.storage.lock().unwrap();
        let left_values = storage_guard.read_column(left_table, left_column, condition.as_ref())?;
        let right_values = storage_guard.read_column(right_table, right_column, condition.as_ref())?;

        let mut column_values = HashMap::new();
        let mut min_row_count_left = usize::MAX;
        let mut min_row_count_right = usize::MAX;
        for col in columns {
            let (table, col_name) = if col.contains('.') {
                let parts = col.split('.').collect::<Vec<_>>();
                (parts[0], parts[1])
            } else {
                (left_table, col.as_str())
            };
            let values = storage_guard.read_column(table, col_name, condition.as_ref())?;
            if table == right_table {
                min_row_count_right = min_row_count_right.min(values.len());
            } else {
                min_row_count_left = min_row_count_left.min(values.len());
            }
            column_values.insert(col.clone(), values);
        }

        // Parallelize the join operation
        let result: Result<Vec<Vec<Value>>, DbError> = (0..min_row_count_left)
            .into_par_iter()
            .flat_map(|i| {
                let left_val = &left_values[i];
                (0..min_row_count_right)
                    .filter_map(|j| {
                        if left_val == &right_values[j] {
                            Some(columns.iter().map(|col| {
                                let values = column_values.get(col).unwrap();
                                let index = if col.starts_with(right_table) { j } else { i };
                                if index < values.len() {
                                    Ok(values[index].clone())
                                } else {
                                    Err(DbError::InvalidData(format!(
                                        "Index {} out of bounds for column {} (len: {})",
                                        index, col, values.len()
                                    )))
                                }
                            }).collect::<Result<Vec<Value>, DbError>>())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        result
    }
}