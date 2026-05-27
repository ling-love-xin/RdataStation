//! Core 内部模型定义
//!
//! 定义 Core 层内部使用的数据模型，包括：
//! - 查询结果结构
//! - 数据值类型
//! - 其他内部数据结构
//!
//! 注意：这些模型会被 api 层重新导出，供前端使用

use arrow::array::*;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;

/// Arrow 批处理类型
pub type ArrowBatch = RecordBatch;

/// 统一的查询结果
#[derive(Debug, Clone, Type)]
pub struct QueryResult {
    pub columns: Vec<String>,
    #[specta(skip)]
    pub batches: Vec<ArrowBatch>,
    /// 影响的行数（对于 INSERT/UPDATE/DELETE）
    pub affected_rows: Option<u32>,
    /// 是否是只读查询（SELECT）
    pub is_read_only: Option<bool>,
}

impl QueryResult {
    /// 创建空的查询结果
    pub fn empty() -> Self {
        Self {
            columns: vec![],
            batches: vec![],
            affected_rows: None,
            is_read_only: None,
        }
    }

    /// 从 Arrow 批处理创建查询结果
    pub fn from_batches(columns: Vec<String>, batches: Vec<ArrowBatch>) -> Self {
        let row_count: u32 = batches.iter().map(|b| b.num_rows() as u32).sum();
        Self {
            columns,
            batches,
            affected_rows: Some(row_count),
            is_read_only: Some(true),
        }
    }

    /// 获取总行数
    pub fn total_rows(&self) -> usize {
        self.batches.iter().map(|b| b.num_rows()).sum()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.batches.is_empty() || self.total_rows() == 0
    }

    /// 截断结果到指定行数（原地操作）
    /// 返回实际截断的行数（0 表示未截断）
    pub fn truncate(&mut self, max_rows: usize) -> usize {
        let total = self.total_rows();
        if total <= max_rows {
            return 0;
        }
        let mut remaining = max_rows;
        self.batches.retain_mut(|batch| {
            if remaining == 0 {
                return false;
            }
            let batch_rows = batch.num_rows();
            if batch_rows <= remaining {
                remaining -= batch_rows;
                true
            } else {
                let truncated_batch = batch.slice(0, remaining);
                *batch = truncated_batch;
                remaining = 0;
                true
            }
        });
        total - max_rows
    }

    /// 将 Arrow batches 转换为行数据（Vec<Vec<Value>>）
    pub fn to_rows(&self) -> Vec<Vec<Value>> {
        let mut rows = Vec::with_capacity(self.total_rows());
        for batch in &self.batches {
            let num_rows = batch.num_rows();
            for row_idx in 0..num_rows {
                let mut row = Vec::with_capacity(self.columns.len());
                for col_idx in 0..batch.num_columns() {
                    row.push(arrow_value_at(batch.column(col_idx), row_idx));
                }
                rows.push(row);
            }
        }
        rows
    }
}

/// 从 Arrow 数组中提取值
fn arrow_value_at(array: &dyn Array, index: usize) -> Value {
    if array.is_null(index) {
        return Value::Null;
    }
    if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
        return Value::Text(arr.value(index).to_string());
    }
    if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
        return Value::Int(arr.value(index));
    }
    if let Some(arr) = array.as_any().downcast_ref::<Float64Array>() {
        return Value::Float(arr.value(index));
    }
    if let Some(arr) = array.as_any().downcast_ref::<BooleanArray>() {
        return Value::Bool(arr.value(index));
    }
    if let Some(arr) = array.as_any().downcast_ref::<BinaryArray>() {
        return Value::Bytes(arr.value(index).to_vec());
    }
    Value::Text(format!("{:?}", array))
}

/// 序列化支持
impl Serialize for QueryResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("QueryResult", 5)?;
        state.serialize_field("columns", &self.columns)?;
        state.serialize_field("rows", &self.to_rows())?;
        state.serialize_field("affected_rows", &self.affected_rows)?;
        state.serialize_field("is_read_only", &self.is_read_only)?;
        state.serialize_field("total_rows", &self.total_rows())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for QueryResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct QueryResultHelper {
            columns: Vec<String>,
            affected_rows: Option<u32>,
            is_read_only: Option<bool>,
        }
        let helper = QueryResultHelper::deserialize(deserializer)?;
        Ok(Self {
            columns: helper.columns,
            batches: vec![],
            affected_rows: helper.affected_rows,
            is_read_only: helper.is_read_only,
        })
    }
}

/// 一行数据（列值数组）
pub type Row = Vec<Value>;

/// 与 SQL 类型兼容的值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{}", v),
            Value::Bytes(v) => write!(f, "{:?}", v),
        }
    }
}

impl Value {
    /// 获取值的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Bool(_) => "BOOLEAN",
            Value::Int(_) => "INTEGER",
            Value::Float(_) => "FLOAT",
            Value::Text(_) => "TEXT",
            Value::Bytes(_) => "BYTES",
        }
    }

    /// 检查是否为 NULL
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// 尝试转换为 i64
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v),
            Value::Float(v) => Some(*v as i64),
            Value::Text(v) => v.parse().ok(),
            _ => None,
        }
    }

    /// 尝试转换为 f64
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Int(v) => Some(*v as f64),
            Value::Text(v) => v.parse().ok(),
            _ => None,
        }
    }

    /// 尝试转换为字符串
    pub fn as_text(&self) -> Option<String> {
        match self {
            Value::Text(v) => Some(v.clone()),
            Value::Int(v) => Some(v.to_string()),
            Value::Float(v) => Some(v.to_string()),
            Value::Bool(v) => Some(v.to_string()),
            Value::Bytes(v) => Some(String::from_utf8_lossy(v).to_string()),
            Value::Null => None,
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v as i64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v as f64)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(val) => val.into(),
            None => Value::Null,
        }
    }
}
