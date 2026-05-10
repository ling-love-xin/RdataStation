use serde::{Deserialize, Serialize};

use crate::core::cache::CacheManager;
use crate::core::error::{CacheError, CommonError, ConnectionError, CoreError};
use crate::core::get_connection_manager;

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
}

#[tauri::command]
pub async fn load_databases(conn_id: String) -> Result<Vec<DatabaseMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_databases(&conn_id))?;
    if let Some(names) = cached {
        return Ok(names
            .into_iter()
            .map(|name| DatabaseMeta { name })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let names = db.list_databases().await?;

    let _ = write_l1_cache(|mc| mc.set_databases(&conn_id, names.clone()));

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
/// 内部委托给 Database trait 的 list_databases() 方法
/// 与 load_databases 共享同一 L1 缓存键
#[tauri::command]
pub async fn load_catalogs(conn_id: String) -> Result<Vec<CatalogMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_databases(&conn_id))?;
    if let Some(databases) = cached {
        return Ok(databases
            .into_iter()
            .map(|name| CatalogMeta { name })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let names = db.list_databases().await?;

    let _ = write_l1_cache(|mc| mc.set_databases(&conn_id, names.clone()));

    Ok(names.into_iter().map(|name| CatalogMeta { name }).collect())
}

#[tauri::command]
pub async fn load_schemas(conn_id: String, db_name: String) -> Result<Vec<SchemaMeta>, CoreError> {
    let cached = check_l1_cache(|mc| mc.get_schemas(&conn_id, &db_name))?;
    if let Some(names) = cached {
        return Ok(names.into_iter().map(|name| SchemaMeta { name }).collect());
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

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let objects = db.list_tables(&db_name, Some(&schema_name)).await?;

    let _ =
        write_l1_cache(|mc| mc.set_tables(&conn_id, &db_name, Some(&schema_name), objects.clone()));

    Ok(objects
        .into_iter()
        .map(|obj| TableMeta {
            name: obj.name,
            table_type: "TABLE".to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn load_views(
    conn_id: String,
    db_name: String,
    schema_name: String,
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

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let objects = db.list_tables(&db_name, Some(&schema_name)).await?;

    let _ =
        write_l1_cache(|mc| mc.set_tables(&conn_id, &db_name, Some(&schema_name), objects.clone()));

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
            })
            .collect());
    }

    let manager = get_connection_manager().clone();
    let db = manager
        .get_connection(&conn_id)
        .await
        .ok_or_else(|| CoreError::connection(ConnectionError::not_found(&conn_id)))?;

    let columns = db
        .list_columns(&db_name, Some(&schema_name), &table_name)
        .await?;

    let _ = write_l1_cache(|mc| {
        mc.set_columns_detail(
            &conn_id,
            &db_name,
            Some(&schema_name),
            &table_name,
            columns.clone(),
        )
    });

    Ok(columns
        .into_iter()
        .map(|col| ColumnMeta {
            name: col.name,
            data_type: col.data_type,
            is_nullable: col.nullable,
            default_value: col.default_value,
            is_primary_key: col.is_primary_key,
        })
        .collect())
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
