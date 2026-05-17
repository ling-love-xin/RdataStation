use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::core::cache::CacheManager;
use crate::core::error::{CacheError, CommonError, ConnectionError, CoreError};
use crate::core::get_connection_manager;
use crate::core::persistence::metadata_cache::{MetadataCacheManager, MetadataCacheOps};

use crate::core::driver::{get_level, remove_level, set_level, IntrospectionLevel};

#[tauri::command]
pub async fn set_introspection_level(
    conn_id: String,
    level: String,
) -> Result<(), CoreError> {
    let il = match level.as_str() {
        "level1" => IntrospectionLevel::Level1,
        "level2" => IntrospectionLevel::Level2,
        "level3" => IntrospectionLevel::Level3,
        _ => {
            return Err(CoreError::common(CommonError::General(format!(
                "Unknown introspection level: {}. Use level1/level2/level3",
                level
            ))))
        }
    };
    set_level(&conn_id, il);
    Ok(())
}

#[tauri::command]
pub async fn get_introspection_level(conn_id: String) -> Result<String, CoreError> {
    Ok(get_level(&conn_id).to_string())
}

#[tauri::command]
pub async fn remove_introspection_level(conn_id: String) -> Result<(), CoreError> {
    remove_level(&conn_id);
    Ok(())
}

static L1_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static L1_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
static L2_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static L2_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
static DB_QUERY_COUNT: AtomicU64 = AtomicU64::new(0);
static L1_HIT_TIME_US: AtomicU64 = AtomicU64::new(0);
static L2_HIT_TIME_US: AtomicU64 = AtomicU64::new(0);
static DB_QUERY_TIME_US: AtomicU64 = AtomicU64::new(0);

fn check_l1_cache<T>(
    get_fn: impl FnOnce(&mut crate::core::cache::MetadataCache) -> Option<T>,
) -> Result<Option<T>, CoreError> {
    let instance = CacheManager::instance();
    let cm = instance
        .lock()
        .map_err(|e| CoreError::cache(CacheError::internal(format!("CacheManager lock: {}", e))))?;
    let mc_arc = cm.metadata_cache();
    let mut mc = mc_arc.lock().map_err(|e| {
        CoreError::cache(CacheError::internal(format!("MetadataCache lock: {}", e)))
    })?;
    Ok(get_fn(&mut mc))
}

fn write_l1_cache(
    set_fn: impl FnOnce(&mut crate::core::cache::MetadataCache),
) -> Result<(), CoreError> {
    let instance = CacheManager::instance();
    let cm = instance
        .lock()
        .map_err(|e| CoreError::cache(CacheError::internal(format!("CacheManager lock: {}", e))))?;
    let mc_arc = cm.metadata_cache();
    let mut mc = mc_arc.lock().map_err(|e| {
        CoreError::cache(CacheError::internal(format!("MetadataCache lock: {}", e)))
    })?;
    set_fn(&mut mc);
    Ok(())
}

/// 检查 L1 缓存命中，若命中则写入新数据（单次锁获取）
///
/// 用于 L2 磁盘缓存命中后回写 L1 内存缓存的场景，
/// 避免 check_l1_cache + write_l1_cache 两次锁获取。
#[allow(dead_code)]
fn l1_check_and_set<T: Clone>(
    _conn_id: &str,
    check_fn: impl FnOnce(&mut crate::core::cache::MetadataCache) -> Option<T>,
    set_value: T,
    set_fn: impl FnOnce(&mut crate::core::cache::MetadataCache, T),
) -> Result<T, CoreError> {
    let instance = CacheManager::instance();
    let cm = instance
        .lock()
        .map_err(|e| CoreError::cache(CacheError::internal(format!("CacheManager lock: {}", e))))?;
    let mc_arc = cm.metadata_cache();
    let mut mc = mc_arc.lock().map_err(|e| {
        CoreError::cache(CacheError::internal(format!("MetadataCache lock: {}", e)))
    })?;
    if let Some(cached) = check_fn(&mut mc) {
        return Ok(cached);
    }
    set_fn(&mut mc, set_value.clone());
    Ok(set_value)
}

fn parse_connection_type(t: &str) -> crate::core::persistence::metadata_cache::ConnectionType {
    match t.to_lowercase().as_str() {
        "project" => crate::core::persistence::metadata_cache::ConnectionType::Project,
        _ => crate::core::persistence::metadata_cache::ConnectionType::Global,
    }
}

fn open_l2_cache(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
) -> Result<MetadataCacheOps, CoreError> {
    let ct = parse_connection_type(connection_type);
    let mgr = MetadataCacheManager::new(conn_id, ct, project_path)?;
    if !mgr.exists() {
        return Err(CoreError::cache(CacheError::internal(
            "L2 disk cache not found".to_string(),
        )));
    }
    let conn = mgr.open()?;
    Ok(MetadataCacheOps::new(conn))
}

fn try_l2_databases(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
) -> Result<Option<Vec<DatabaseMeta>>, CoreError> {
    let ops = open_l2_cache(conn_id, connection_type, project_path)?;
    let conn = ops.get_connection();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT catalog_name FROM schemata WHERE catalog_name IS NOT NULL ORDER BY catalog_name",
    )
    .map_err(|e| CoreError::cache(CacheError::internal(format!("L2 databases query: {}", e))))?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| CoreError::cache(CacheError::internal(format!("L2 databases rows: {}", e))))?
        .filter_map(|r| r.ok())
        .collect();
    if names.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            names
                .into_iter()
                .map(|name| DatabaseMeta { name })
                .collect(),
        ))
    }
}

fn try_l2_schemas(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
    database: &str,
) -> Result<Option<Vec<SchemaMeta>>, CoreError> {
    let ops = open_l2_cache(conn_id, connection_type, project_path)?;
    let schema_infos = ops
        .list_schemas(Some(database))
        .map_err(|e| CoreError::cache(CacheError::internal(format!("L2 schemas list: {}", e))))?;
    if schema_infos.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            schema_infos
                .into_iter()
                .map(|s| SchemaMeta {
                    name: s.schema_name,
                })
                .collect(),
        ))
    }
}

fn try_l2_tables(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
    database: &str,
    schema_name: &str,
) -> Result<Option<Vec<TableMeta>>, CoreError> {
    let ops = open_l2_cache(conn_id, connection_type, project_path)?;
    let conn = ops.get_connection();
    let mut stmt = conn
        .prepare(
            "SELECT t.table_name, t.table_type FROM tables t
         INNER JOIN schemata s ON t.schema_id = s.id
         WHERE s.catalog_name = ?1 AND s.schema_name = ?2
         ORDER BY t.table_name",
        )
        .map_err(|e| CoreError::cache(CacheError::internal(format!("L2 tables query: {}", e))))?;
    let tables: Vec<TableMeta> = stmt
        .query_map(rusqlite::params![database, schema_name], |row| {
            Ok(TableMeta {
                name: row.get(0)?,
                table_type: row
                    .get::<_, String>(1)
                    .unwrap_or_else(|_| "TABLE".to_string()),
            })
        })
        .map_err(|e| CoreError::cache(CacheError::internal(format!("L2 tables rows: {}", e))))?
        .filter_map(|r| r.ok())
        .collect();
    if tables.is_empty() {
        Ok(None)
    } else {
        Ok(Some(tables))
    }
}

fn try_l2_columns(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
    database: &str,
    schema_name: &str,
    table_name: &str,
) -> Result<Option<Vec<ColumnMeta>>, CoreError> {
    let ops = open_l2_cache(conn_id, connection_type, project_path)?;
    let column_infos = ops
        .list_columns(database, schema_name, table_name)
        .map_err(|e| CoreError::cache(CacheError::internal(format!("L2 columns list: {}", e))))?;
    if column_infos.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            column_infos
                .into_iter()
                .map(|c| ColumnMeta {
                    name: c.name,
                    data_type: c.data_type,
                    is_nullable: c.is_nullable,
                    default_value: None,
                    is_primary_key: c.is_primary,
                    is_foreign_key: false,
                    comment: c.comment,
                })
                .collect(),
        ))
    }
}

fn write_l2_tables_after_load(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
    database: &str,
    schema_name: &str,
    tables: &[TableMeta],
) {
    if let Ok(mut ops) = open_l2_cache_for_write(conn_id, connection_type, project_path) {
        let batch: Vec<(String, String, String, String, Option<String>)> = tables
            .iter()
            .map(|t| {
                (
                    database.to_string(),
                    schema_name.to_string(),
                    t.name.clone(),
                    t.table_type.clone(),
                    None::<String>,
                )
            })
            .collect();
        let _ = ops.save_tables_batch(batch);
    }
}

fn write_l2_columns_after_load(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
    database: &str,
    schema_name: &str,
    table_name: &str,
    columns: &[ColumnMeta],
) {
    type ColumnBatchEntry = (
        String,
        String,
        String,
        String,
        String,
        String,
        bool,
        bool,
        bool,
    );
    if let Ok(mut ops) = open_l2_cache_for_write(conn_id, connection_type, project_path) {
        let batch: Vec<ColumnBatchEntry> = columns
            .iter()
            .map(|c| {
                (
                    database.to_string(),
                    schema_name.to_string(),
                    table_name.to_string(),
                    c.name.clone(),
                    c.data_type.clone(),
                    if c.is_nullable {
                        "YES".to_string()
                    } else {
                        "NO".to_string()
                    },
                    c.is_primary_key,
                    c.is_foreign_key,
                    false,
                )
            })
            .collect();
        let _ = ops.save_columns_batch(batch);
    }
}

fn open_l2_cache_for_write(
    conn_id: &str,
    connection_type: &str,
    project_path: Option<&str>,
) -> Result<MetadataCacheOps, CoreError> {
    let ct = parse_connection_type(connection_type);
    let mgr = MetadataCacheManager::new(conn_id, ct, project_path)?;
    let conn = mgr.open()?;
    Ok(MetadataCacheOps::new(conn))
}

#[tauri::command]
pub async fn invalidate_metadata_cache(conn_id: String) -> Result<(), CoreError> {
    let instance = CacheManager::instance();
    let cm = instance
        .lock()
        .map_err(|e| CoreError::cache(CacheError::internal(format!("CacheManager lock: {}", e))))?;
    cm.invalidate_connection(&conn_id);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub table_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub view_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    #[serde(rename = "isNullable")]
    pub is_nullable: bool,
    #[serde(rename = "defaultValue")]
    pub default_value: Option<String>,
    #[serde(rename = "isPrimaryKey")]
    pub is_primary_key: bool,
    #[serde(rename = "isForeignKey")]
    pub is_foreign_key: bool,
    #[serde(rename = "comment")]
    pub comment: Option<String>,
}

#[tauri::command]
pub async fn load_databases(
    conn_id: String,
    connection_type: Option<String>,
    project_path: Option<String>,
) -> Result<Vec<DatabaseMeta>, CoreError> {
    let t0 = Instant::now();
    let cached = check_l1_cache(|mc| mc.get_catalogs(&conn_id))?;
    if let Some(names) = cached {
        L1_HIT_COUNT.fetch_add(1, Ordering::Relaxed);
        L1_HIT_TIME_US.fetch_add(t0.elapsed().as_micros() as u64, Ordering::Relaxed);
        return Ok(names
            .into_iter()
            .map(|name| DatabaseMeta { name })
            .collect());
    }
    L1_MISS_COUNT.fetch_add(1, Ordering::Relaxed);

    let ct = connection_type.as_deref().unwrap_or("global");
    let pp = project_path.as_deref();
    let t1 = Instant::now();
    if let Ok(Some(databases)) = try_l2_databases(&conn_id, ct, pp) {
        L2_HIT_COUNT.fetch_add(1, Ordering::Relaxed);
        L2_HIT_TIME_US.fetch_add(t1.elapsed().as_micros() as u64, Ordering::Relaxed);
        let names: Vec<String> = databases.iter().map(|d| d.name.clone()).collect();
        let _ = write_l1_cache(|mc| mc.set_catalogs(&conn_id, names));
        return Ok(databases);
    }
    L2_MISS_COUNT.fetch_add(1, Ordering::Relaxed);

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let t2 = Instant::now();
    let names = db.list_catalogs().await?;
    DB_QUERY_COUNT.fetch_add(1, Ordering::Relaxed);
    DB_QUERY_TIME_US.fetch_add(t2.elapsed().as_micros() as u64, Ordering::Relaxed);

    let _ = write_l1_cache(|mc| mc.set_catalogs(&conn_id, names.clone()));

    Ok(names
        .into_iter()
        .map(|name| DatabaseMeta { name })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogMeta {
    pub name: String,
}

/// ANSI SQL 标准：Catalog（目录）是顶层容器，包含多个 Schema
/// 内部委托给 Database trait 的 list_catalogs() 方法
/// 与 load_databases 共享同一 L1 缓存键
#[tauri::command]
pub async fn load_catalogs(
    conn_id: String,
    connection_type: Option<String>,
    project_path: Option<String>,
) -> Result<Vec<CatalogMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_catalogs(&conn_id))?;
    if let Some(databases) = cached {
        return Ok(databases
            .into_iter()
            .map(|name| CatalogMeta { name })
            .collect());
    }

    let ct = connection_type.as_deref().unwrap_or("global");
    let pp = project_path.as_deref();
    if let Ok(Some(databases)) = try_l2_databases(&conn_id, ct, pp) {
        let names: Vec<String> = databases.iter().map(|d| d.name.clone()).collect();
        let _ = write_l1_cache(|mc| mc.set_catalogs(&conn_id, names));
        return Ok(databases
            .into_iter()
            .map(|d| CatalogMeta { name: d.name })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let names = db.list_catalogs().await?;

    let _ = write_l1_cache(|mc| mc.set_catalogs(&conn_id, names.clone()));

    Ok(names.into_iter().map(|name| CatalogMeta { name }).collect())
}

#[tauri::command]
pub async fn load_schemas(
    conn_id: String,
    db_name: String,
    connection_type: Option<String>,
    project_path: Option<String>,
) -> Result<Vec<SchemaMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_schemas(&conn_id, &db_name))?;
    if let Some(names) = cached {
        return Ok(names.into_iter().map(|name| SchemaMeta { name }).collect());
    }

    let ct = connection_type.as_deref().unwrap_or("global");
    let pp = project_path.as_deref();
    if let Ok(Some(schemas)) = try_l2_schemas(&conn_id, ct, pp, &db_name) {
        let names: Vec<String> = schemas.iter().map(|s| s.name.clone()).collect();
        let _ = write_l1_cache(|mc| mc.set_schemas(&conn_id, &db_name, names));
        return Ok(schemas);
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let names = db.list_schemas(&db_name).await?;

    let _ = write_l1_cache(|mc| mc.set_schemas(&conn_id, &db_name, names.clone()));

    Ok(names.into_iter().map(|name| SchemaMeta { name }).collect())
}

#[tauri::command]
pub async fn load_tables(
    conn_id: String,
    db_name: String,
    schema_name: String,
    connection_type: Option<String>,
    project_path: Option<String>,
) -> Result<Vec<TableMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_tables(&conn_id, &db_name, Some(&schema_name)))?;
    if let Some(objects) = cached {
        return Ok(objects
            .into_iter()
            .map(|obj| TableMeta {
                name: obj.name,
                table_type: "TABLE".to_string(),
            })
            .collect());
    }

    let ct = connection_type.as_deref().unwrap_or("global");
    let pp = project_path.as_deref();
    if let Ok(Some(tables)) = try_l2_tables(&conn_id, ct, pp, &db_name, &schema_name) {
        let objects: Vec<crate::core::driver::SchemaObject> = tables
            .iter()
            .map(|t| crate::core::driver::SchemaObject {
                name: t.name.clone(),
                kind: crate::core::driver::SchemaObjectKind::Table,
                children: None,
                comment: None,
            })
            .collect();
        let _ = write_l1_cache(|mc| mc.set_tables(&conn_id, &db_name, Some(&schema_name), objects));
        return Ok(tables);
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let objects = db.list_tables(&db_name, Some(&schema_name)).await?;

    let _ =
        write_l1_cache(|mc| mc.set_tables(&conn_id, &db_name, Some(&schema_name), objects.clone()));

    let table_metas: Vec<TableMeta> = objects
        .iter()
        .map(|obj| TableMeta {
            name: obj.name.clone(),
            table_type: "TABLE".to_string(),
        })
        .collect();
    write_l2_tables_after_load(&conn_id, ct, pp, &db_name, &schema_name, &table_metas);

    Ok(table_metas)
}

#[tauri::command]
pub async fn load_views(
    conn_id: String,
    db_name: String,
    schema_name: String,
    connection_type: Option<String>,
    project_path: Option<String>,
) -> Result<Vec<ViewMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_tables(&conn_id, &db_name, Some(&schema_name)))?;
    if let Some(objects) = cached {
        return Ok(objects
            .into_iter()
            .filter(|obj| matches!(obj.kind, crate::core::driver::SchemaObjectKind::View))
            .map(|obj| ViewMeta {
                name: obj.name,
                view_type: "VIEW".to_string(),
            })
            .collect());
    }

    let ct = connection_type.as_deref().unwrap_or("global");
    let pp = project_path.as_deref();
    if let Ok(Some(tables)) = try_l2_tables(&conn_id, ct, pp, &db_name, &schema_name) {
        let objects: Vec<crate::core::driver::SchemaObject> = tables
            .iter()
            .map(|t| crate::core::driver::SchemaObject {
                name: t.name.clone(),
                kind: if t.table_type.to_uppercase() == "VIEW" {
                    crate::core::driver::SchemaObjectKind::View
                } else {
                    crate::core::driver::SchemaObjectKind::Table
                },
                children: None,
                comment: None,
            })
            .collect();
        let _ = write_l1_cache(|mc| mc.set_tables(&conn_id, &db_name, Some(&schema_name), objects));
        return Ok(tables
            .into_iter()
            .filter(|t| t.table_type.to_uppercase() == "VIEW")
            .map(|t| ViewMeta {
                name: t.name,
                view_type: "VIEW".to_string(),
            })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let objects = db.list_tables(&db_name, Some(&schema_name)).await?;

    let _ =
        write_l1_cache(|mc| mc.set_tables(&conn_id, &db_name, Some(&schema_name), objects.clone()));

    let table_metas: Vec<TableMeta> = objects
        .iter()
        .map(|obj| TableMeta {
            name: obj.name.clone(),
            table_type: if matches!(obj.kind, crate::core::driver::SchemaObjectKind::View) {
                "VIEW".to_string()
            } else {
                "TABLE".to_string()
            },
        })
        .collect();
    write_l2_tables_after_load(&conn_id, ct, pp, &db_name, &schema_name, &table_metas);

    Ok(objects
        .into_iter()
        .filter(|obj| matches!(obj.kind, crate::core::driver::SchemaObjectKind::View))
        .map(|obj| ViewMeta {
            name: obj.name,
            view_type: "VIEW".to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn load_columns(
    conn_id: String,
    db_name: String,
    schema_name: String,
    table_name: String,
    connection_type: Option<String>,
    project_path: Option<String>,
) -> Result<Vec<ColumnMeta>, CoreError> {
    let cached = check_l1_cache(|mc| {
        mc.get_columns_detail(&conn_id, &db_name, Some(&schema_name), &table_name)
    })?;
    if let Some(columns) = cached {
        return Ok(columns
            .into_iter()
            .map(|col| ColumnMeta {
                name: col.name,
                data_type: col.data_type,
                is_nullable: col.nullable,
                default_value: col.default_value,
                is_primary_key: col.is_primary_key,
                is_foreign_key: col.is_foreign_key,
                comment: col.comment,
            })
            .collect());
    }

    let ct = connection_type.as_deref().unwrap_or("global");
    let pp = project_path.as_deref();
    if let Ok(Some(columns)) = try_l2_columns(&conn_id, ct, pp, &db_name, &schema_name, &table_name)
    {
        let details: Vec<crate::core::driver::ColumnDetail> = columns
            .iter()
            .map(|c| crate::core::driver::ColumnDetail {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.is_nullable,
                is_primary_key: c.is_primary_key,
                is_foreign_key: c.is_foreign_key,
                default_value: c.default_value.clone(),
                comment: c.comment.clone(),
            })
            .collect();
        let _ = write_l1_cache(|mc| {
            mc.set_columns_detail(&conn_id, &db_name, Some(&schema_name), &table_name, details)
        });
        return Ok(columns);
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let columns_detail = db
        .list_columns(&db_name, Some(&schema_name), &table_name)
        .await?;

    let column_metas: Vec<ColumnMeta> = columns_detail
        .iter()
        .map(|col| ColumnMeta {
            name: col.name.clone(),
            data_type: col.data_type.clone(),
            is_nullable: col.nullable,
            default_value: col.default_value.clone(),
            is_primary_key: col.is_primary_key,
            is_foreign_key: col.is_foreign_key,
            comment: col.comment.clone(),
        })
        .collect();

    let _ = write_l1_cache(|mc| {
        mc.set_columns_detail(
            &conn_id,
            &db_name,
            Some(&schema_name),
            &table_name,
            columns_detail,
        )
    });

    write_l2_columns_after_load(
        &conn_id,
        ct,
        pp,
        &db_name,
        &schema_name,
        &table_name,
        &column_metas,
    );

    Ok(column_metas)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMeta {
    pub name: String,
}

#[tauri::command]
pub async fn load_procedures(
    conn_id: String,
    db_name: String,
    schema_name: String,
) -> Result<Vec<ProcedureMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_procedures(&conn_id, &db_name, Some(&schema_name)))?;
    if let Some(objects) = cached {
        return Ok(objects
            .into_iter()
            .map(|obj| ProcedureMeta { name: obj.name })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let objects = db.list_procedures(&db_name, Some(&schema_name)).await?;

    let _ = write_l1_cache(|mc| {
        mc.set_procedures(&conn_id, &db_name, Some(&schema_name), objects.clone())
    });

    Ok(objects
        .into_iter()
        .map(|obj| ProcedureMeta { name: obj.name })
        .collect())
}

#[tauri::command]
pub async fn load_functions(
    conn_id: String,
    db_name: String,
    schema_name: String,
) -> Result<Vec<FunctionMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_functions(&conn_id, &db_name, Some(&schema_name)))?;
    if let Some(objects) = cached {
        return Ok(objects
            .into_iter()
            .map(|obj| FunctionMeta { name: obj.name })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let objects = db.list_functions(&db_name, Some(&schema_name)).await?;

    let _ = write_l1_cache(|mc| {
        mc.set_functions(&conn_id, &db_name, Some(&schema_name), objects.clone())
    });

    Ok(objects
        .into_iter()
        .map(|obj| FunctionMeta { name: obj.name })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineSourceMeta {
    pub name: String,
    #[serde(rename = "routineKind")]
    pub routine_kind: String,
    #[serde(rename = "sourceCode")]
    pub source_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub db_queries: u64,
    pub l1_hit_avg_us: u64,
    pub l2_hit_avg_us: u64,
    pub db_query_avg_us: u64,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub overall_hit_rate: f64,
}

impl CacheStats {
    fn snapshot() -> Self {
        let l1_hits = L1_HIT_COUNT.load(Ordering::Relaxed);
        let l1_misses = L1_MISS_COUNT.load(Ordering::Relaxed);
        let l2_hits = L2_HIT_COUNT.load(Ordering::Relaxed);
        let l2_misses = L2_MISS_COUNT.load(Ordering::Relaxed);
        let db_queries = DB_QUERY_COUNT.load(Ordering::Relaxed);
        let l1_hit_time = L1_HIT_TIME_US.load(Ordering::Relaxed);
        let l2_hit_time = L2_HIT_TIME_US.load(Ordering::Relaxed);
        let db_query_time = DB_QUERY_TIME_US.load(Ordering::Relaxed);

        let l1_total = l1_hits + l1_misses;
        let l2_total = l2_hits + l2_misses;

        Self {
            l1_hits,
            l1_misses,
            l2_hits,
            l2_misses,
            db_queries,
            l1_hit_avg_us: l1_hit_time.checked_div(l1_hits).unwrap_or(0),
            l2_hit_avg_us: l2_hit_time.checked_div(l2_hits).unwrap_or(0),
            db_query_avg_us: db_query_time.checked_div(db_queries).unwrap_or(0),
            l1_hit_rate: if l1_total > 0 {
                l1_hits as f64 / l1_total as f64
            } else {
                0.0
            },
            l2_hit_rate: if l2_total > 0 {
                l2_hits as f64 / l2_total as f64
            } else {
                0.0
            },
            overall_hit_rate: if l1_total > 0 {
                (l1_hits + l2_hits) as f64 / l1_total as f64
            } else {
                0.0
            },
        }
    }
}

#[tauri::command]
pub async fn get_cache_stats() -> Result<CacheStats, CoreError> {
    Ok(CacheStats::snapshot())
}

#[tauri::command]
pub async fn reset_cache_stats() -> Result<(), CoreError> {
    L1_HIT_COUNT.store(0, Ordering::Relaxed);
    L1_MISS_COUNT.store(0, Ordering::Relaxed);
    L2_HIT_COUNT.store(0, Ordering::Relaxed);
    L2_MISS_COUNT.store(0, Ordering::Relaxed);
    DB_QUERY_COUNT.store(0, Ordering::Relaxed);
    L1_HIT_TIME_US.store(0, Ordering::Relaxed);
    L2_HIT_TIME_US.store(0, Ordering::Relaxed);
    DB_QUERY_TIME_US.store(0, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn load_routine_source(
    conn_id: String,
    db_name: String,
    schema_name: String,
    routine_name: String,
    routine_kind: String,
) -> Result<RoutineSourceMeta, CoreError> {
    let kind_str = routine_kind.clone();
    let cached = check_l1_cache(|mc| {
        mc.get_routine_source(
            &conn_id,
            &db_name,
            Some(&schema_name),
            &routine_name,
            &routine_kind,
        )
    })?;
    if let Some(source) = cached {
        return Ok(RoutineSourceMeta {
            name: routine_name,
            routine_kind: kind_str,
            source_code: Some(source),
        });
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let kind = match routine_kind.as_str() {
        "procedure" => crate::core::driver::SchemaObjectKind::Procedure,
        "function" => crate::core::driver::SchemaObjectKind::Function,
        _ => {
            return Err(CoreError::common(CommonError::invalid_argument(
                "routine_kind",
                format!(
                    "Unknown routine kind: {}. Expected 'procedure' or 'function'",
                    routine_kind
                ),
            )))
        }
    };

    let source = db
        .get_routine_source(&db_name, Some(&schema_name), &routine_name, kind)
        .await?;

    if let Some(ref src) = source {
        let _ = write_l1_cache(|mc| {
            mc.set_routine_source(
                &conn_id,
                &db_name,
                Some(&schema_name),
                &routine_name,
                &routine_kind,
                src.clone(),
            )
        });
    }

    Ok(RoutineSourceMeta {
        name: routine_name,
        routine_kind,
        source_code: source,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMeta {
    pub name: String,
    pub table_name: String,
    pub column_names: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}

#[tauri::command]
pub async fn load_indexes(
    conn_id: String,
    table_name: String,
) -> Result<Vec<IndexMeta>, CoreError> {
    tracing::debug!("load_indexes called for conn={conn_id}, table={table_name} (not yet implemented)");
    Ok(vec![])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintMeta {
    pub name: String,
    pub table_name: String,
    pub constraint_type: String,
    pub column_names: Vec<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
}

#[tauri::command]
pub async fn load_constraints(
    conn_id: String,
    table_name: String,
) -> Result<Vec<ConstraintMeta>, CoreError> {
    tracing::debug!("load_constraints called for conn={conn_id}, table={table_name} (not yet implemented)");
    Ok(vec![])
}
