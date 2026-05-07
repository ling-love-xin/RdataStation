use std::collections::HashMap;

use duckdb::Connection;
use serde_json::{json, Value};

use crate::core::error::{CommonError, CoreError};

use super::rule_types::RuleFile;

pub struct RuleExecutor;

impl RuleExecutor {
    pub fn execute(
        rule: &RuleFile,
        conn: &Connection,
        params: &HashMap<String, String>,
    ) -> Result<Value, CoreError> {
        let sql = Self::build_sql(rule, params)?;
        Self::validate_identifiers(params)?;

        let result_type = rule.query.result_type.as_deref().unwrap_or("single");

        match result_type {
            "list" => Self::execute_list(rule, conn, &sql),
            _ => Self::execute_single(rule, conn, &sql),
        }
    }

    fn build_sql(rule: &RuleFile, params: &HashMap<String, String>) -> Result<String, CoreError> {
        let mut sql = rule.query.template.clone();

        for param_name in &rule.query.parameters {
            let placeholder = format!("{{{}}}", param_name);
            let value = params.get(param_name).ok_or_else(|| {
                CoreError::common(CommonError::General(format!(
                    "Missing parameter '{}' for rule '{}'",
                    param_name, rule.meta.id
                )))
            })?;
            sql = sql.replace(&placeholder, value);
        }

        if sql.contains('{') {
            let missing: Vec<String> = rule
                .query
                .parameters
                .iter()
                .filter(|p| !params.contains_key(*p))
                .map(|p| p.clone())
                .collect();
            if !missing.is_empty() {
                return Err(CoreError::common(CommonError::General(format!(
                    "Missing parameters for rule '{}': {:?}",
                    rule.meta.id, missing
                ))));
            }
        }

        Ok(sql)
    }

    fn validate_identifiers(params: &HashMap<String, String>) -> Result<(), CoreError> {
        for (key, value) in params {
            if !value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
            {
                return Err(CoreError::common(CommonError::General(format!(
                    "Invalid characters in parameter '{}' value: '{}'",
                    key, value
                ))));
            }
        }
        Ok(())
    }

    fn execute_single(
        rule: &RuleFile,
        conn: &Connection,
        sql: &str,
    ) -> Result<Value, CoreError> {
        let mut stmt = conn.prepare(sql).map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "Rule '{}' prepare failed: {}",
                rule.meta.id, e
            )))
        })?;

        let mut map = serde_json::Map::new();

        let has_row = stmt
            .query_map([], |row| {
                for field in &rule.output {
                    let val: Value = match field.value_type.as_str() {
                        "f64" => {
                            let v: f64 = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "f64?" => {
                            let v: Option<f64> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(x) => json!(x),
                                None => Value::Null,
                            }
                        }
                        "i64" => {
                            let v: i64 = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "i64?" => {
                            let v: Option<i64> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(x) => json!(x),
                                None => Value::Null,
                            }
                        }
                        "String" | "string" => {
                            let v: String =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "String?" | "string?" => {
                            let v: Option<String> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(s) => json!(s),
                                None => Value::Null,
                            }
                        }
                        "bool" => {
                            let v: bool = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "bool?" => {
                            let v: Option<bool> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(b) => json!(b),
                                None => Value::Null,
                            }
                        }
                        "usize" => {
                            let v: i64 = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v as usize)
                        }
                        _ => {
                            let v: String =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                    };
                    map.insert(field.json_name.clone(), val);
                }
                Ok(())
            })
            .map_err(|e| {
                CoreError::common(CommonError::General(format!(
                    "Rule '{}' query failed: {}",
                    rule.meta.id, e
                )))
            })?;

        let mut found = false;
        for _row in has_row {
            found = true;
        }

        if found {
            Ok(Value::Object(map))
        } else {
            Ok(Value::Null)
        }
    }

    fn execute_list(
        rule: &RuleFile,
        conn: &Connection,
        sql: &str,
    ) -> Result<Value, CoreError> {
        let mut stmt = conn.prepare(sql).map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "Rule '{}' prepare failed: {}",
                rule.meta.id, e
            )))
        })?;

        let rows: Vec<Value> = stmt
            .query_map([], |row| {
                let mut map = serde_json::Map::new();
                for field in &rule.output {
                    let val: Value = match field.value_type.as_str() {
                        "f64" => {
                            let v: f64 = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "f64?" => {
                            let v: Option<f64> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(x) => json!(x),
                                None => Value::Null,
                            }
                        }
                        "i64" => {
                            let v: i64 = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "i64?" => {
                            let v: Option<i64> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(x) => json!(x),
                                None => Value::Null,
                            }
                        }
                        "String" | "string" => {
                            let v: String =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "String?" | "string?" => {
                            let v: Option<String> =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            match v {
                                Some(s) => json!(s),
                                None => Value::Null,
                            }
                        }
                        "bool" => {
                            let v: bool = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                        "usize" => {
                            let v: i64 = row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v as usize)
                        }
                        _ => {
                            let v: String =
                                row.get_unwrap(Self::col_index(row, &field.sql_name));
                            json!(v)
                        }
                    };
                    map.insert(field.json_name.clone(), val);
                }
                Ok(Value::Object(map))
            })
            .map_err(|e| {
                CoreError::common(CommonError::General(format!(
                    "Rule '{}' list query failed: {}",
                    rule.meta.id, e
                )))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Value::Array(rows))
    }

    fn col_index(row: &duckdb::Row, col_name: &str) -> usize {
        for i in 0.. {
            if let Ok(name) = row.as_ref().column_name(i) {
                if name.eq_ignore_ascii_case(col_name) {
                    return i;
                }
            } else {
                break;
            }
        }
        panic!("Column '{}' not found in result", col_name)
    }
}
