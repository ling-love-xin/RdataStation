//! SQL 模板相关命令
//!
//! 处理 SQL 模板的创建、查询、删除等操作

use crate::core::migration::global_init;
use crate::core::persistence::SqlTemplate;

/// SQL 模板响应
#[derive(serde::Serialize, Debug)]
pub struct SqlTemplateResponse {
    pub id: String,
    pub name: String,
    pub content: String,
    pub db_type: Option<String>,
    pub category: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub is_builtin: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl From<SqlTemplate> for SqlTemplateResponse {
    fn from(template: SqlTemplate) -> Self {
        Self {
            id: template.id,
            name: template.name,
            content: template.content,
            db_type: template.db_type,
            category: template.category,
            description: template.description,
            tags: template.tags,
            is_builtin: template.is_builtin,
            created_at_ms: template.created_at_ms,
            updated_at_ms: template.updated_at_ms,
        }
    }
}

/// 创建 SQL 模板请求参数
#[derive(serde::Deserialize, Debug)]
pub struct CreateSqlTemplateInput {
    pub name: String,
    pub content: String,
    pub db_type: Option<String>,
    pub category: String,
    pub description: Option<String>,
    pub tags: Option<String>,
}

/// 创建 SQL 模板
#[tauri::command]
pub async fn create_sql_template(
    input: CreateSqlTemplateInput,
) -> Result<SqlTemplateResponse, String> {
    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let template = SqlTemplate::new(
        input.name,
        input.content,
        input.db_type,
        input.category,
        input.description,
        input.tags,
    );

    let store = global_db
        .get_sql_template_store()
        .map_err(|e| format!("获取模板存储失败: {}", e))?;

    store
        .save(&template)
        .map_err(|e| format!("保存模板失败: {}", e))?;

    Ok(template.into())
}

/// 获取所有 SQL 模板
#[tauri::command]
pub async fn get_all_sql_templates() -> Result<Vec<SqlTemplateResponse>, String> {
    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let store = global_db
        .get_sql_template_store()
        .map_err(|e| format!("获取模板存储失败: {}", e))?;

    let templates = store
        .get_all()
        .map_err(|e| format!("获取模板列表失败: {}", e))?;

    Ok(templates.into_iter().map(|t| t.into()).collect())
}

/// 根据分类获取 SQL 模板
#[tauri::command]
pub async fn get_sql_templates_by_category(
    category: String,
) -> Result<Vec<SqlTemplateResponse>, String> {
    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let store = global_db
        .get_sql_template_store()
        .map_err(|e| format!("获取模板存储失败: {}", e))?;

    let templates = store
        .get_by_category(&category)
        .map_err(|e| format!("获取模板列表失败: {}", e))?;

    Ok(templates.into_iter().map(|t| t.into()).collect())
}

/// 根据数据库类型获取 SQL 模板
#[tauri::command]
pub async fn get_sql_templates_by_db_type(
    db_type: String,
) -> Result<Vec<SqlTemplateResponse>, String> {
    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let store = global_db
        .get_sql_template_store()
        .map_err(|e| format!("获取模板存储失败: {}", e))?;

    let templates = store
        .get_by_db_type(&db_type)
        .map_err(|e| format!("获取模板列表失败: {}", e))?;

    Ok(templates.into_iter().map(|t| t.into()).collect())
}

/// 删除 SQL 模板
#[tauri::command]
pub async fn delete_sql_template(template_id: String) -> Result<bool, String> {
    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let store = global_db
        .get_sql_template_store()
        .map_err(|e| format!("获取模板存储失败: {}", e))?;

    store
        .delete(&template_id)
        .map_err(|e| format!("删除模板失败: {}", e))
}

/// 获取所有 SQL 模板分类
#[tauri::command]
pub async fn get_sql_template_categories() -> Result<Vec<String>, String> {
    let global_db = global_init::get_global_db_manager()
        .ok_or_else(|| "Global database manager not initialized".to_string())?;

    let store = global_db
        .get_sql_template_store()
        .map_err(|e| format!("获取模板存储失败: {}", e))?;

    store
        .get_categories()
        .map_err(|e| format!("获取分类列表失败: {}", e))
}
