//! ID 前缀规范模块
//!
//! 提供全局（Application）与项目（Project）级别主键的 ID 前缀常量、
//! 生成函数和前缀检测工具。
//!
//! ## 前缀约定
//!
//! | 前缀  | 含义                             | 存储位置        |
//! |-------|----------------------------------|-----------------|
//! | `G_`  | 全局（Application）表主键        | global.db       |
//! | `P_`  | 项目（Project）表主键（本地创建）| project.db      |
//! | `GP_` | 从全局快照到项目的数据           | project.db      |

/// 全局（Application）表 ID 前缀
pub const PREFIX_GLOBAL: &str = "G_";

/// 项目（Project）表本地 ID 前缀
pub const PREFIX_PROJECT: &str = "P_";

/// 全局快照到项目的 ID 前缀
pub const PREFIX_GLOBAL_SNAPSHOT: &str = "GP_";

/// 快照来源标识
pub const ORIGIN_PROJECT: &str = "project";
pub const ORIGIN_GLOBAL_SNAPSHOT: &str = "global_snapshot";

/// 生成全局 ID
///
/// # Arguments
/// * `kind` - 实体类型，如 "env", "net", "auth", "ep"
/// * `uuid` - UUID v4 字符串
///
/// # Examples
/// ```
/// let id = gen_global_id("env", "a1b2c3d4");
/// assert_eq!(id, "G_env_a1b2c3d4");
/// ```
pub fn gen_global_id(kind: &str, uuid: &str) -> String {
    format!("{}{}_{}", PREFIX_GLOBAL, kind, uuid)
}

/// 生成项目本地 ID
pub fn gen_project_id(kind: &str, uuid: &str) -> String {
    format!("{}{}_{}", PREFIX_PROJECT, kind, uuid)
}

/// 从全局 ID 生成快照 ID（G_env_dev → GP_env_dev）
pub fn to_snapshot_id(global_id: &str) -> Option<String> {
    global_id
        .strip_prefix(PREFIX_GLOBAL)
        .map(|suffix| format!("{}{}", PREFIX_GLOBAL_SNAPSHOT, suffix))
}

/// 检测 ID 是否为全局前缀
pub fn is_global(id: &str) -> bool {
    id.starts_with(PREFIX_GLOBAL) && !id.starts_with(PREFIX_GLOBAL_SNAPSHOT)
}

/// 检测 ID 是否为项目本地前缀
pub fn is_project(id: &str) -> bool {
    id.starts_with(PREFIX_PROJECT)
}

/// 检测 ID 是否为全局快照前缀
pub fn is_global_snapshot(id: &str) -> bool {
    id.starts_with(PREFIX_GLOBAL_SNAPSHOT)
}

/// 获取 ID 前缀类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdPrefix {
    Global,
    Project,
    GlobalSnapshot,
    Unknown,
}

/// 检测 ID 前缀类型
pub fn classify(id: &str) -> IdPrefix {
    if id.starts_with(PREFIX_GLOBAL_SNAPSHOT) {
        IdPrefix::GlobalSnapshot
    } else if id.starts_with(PREFIX_GLOBAL) {
        IdPrefix::Global
    } else if id.starts_with(PREFIX_PROJECT) {
        IdPrefix::Project
    } else {
        IdPrefix::Unknown
    }
}

/// 从快照 ID 反查全局源 ID（GP_env_dev → G_env_dev）
pub fn source_global_id(snapshot_id: &str) -> Option<String> {
    snapshot_id
        .strip_prefix(PREFIX_GLOBAL_SNAPSHOT)
        .map(|suffix| format!("{}{}", PREFIX_GLOBAL, suffix))
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_global_id() {
        let id = gen_global_id("env", "abc123");
        assert_eq!(id, "G_env_abc123");
    }

    #[test]
    fn test_gen_project_id() {
        let id = gen_project_id("net", "xyz789");
        assert_eq!(id, "P_net_xyz789");
    }

    #[test]
    fn test_to_snapshot_id() {
        let sn = to_snapshot_id("G_env_dev");
        assert_eq!(sn, Some("GP_env_dev".to_string()));

        assert_eq!(to_snapshot_id("P_env_dev"), None);
        assert_eq!(to_snapshot_id("GP_env_dev"), None);
    }

    #[test]
    fn test_source_global_id() {
        let src = source_global_id("GP_env_dev");
        assert_eq!(src, Some("G_env_dev".to_string()));

        assert_eq!(source_global_id("G_env_dev"), None);
        assert_eq!(source_global_id("P_env_dev"), None);
    }

    #[test]
    fn test_classify() {
        assert_eq!(classify("G_env_prod"), IdPrefix::Global);
        assert_eq!(classify("P_env_custom"), IdPrefix::Project);
        assert_eq!(classify("GP_env_prod"), IdPrefix::GlobalSnapshot);
        assert_eq!(classify("env_dev"), IdPrefix::Unknown);
        assert_eq!(classify("legacy_id"), IdPrefix::Unknown);
    }

    #[test]
    fn test_is_functions() {
        assert!(is_global("G_env_prod"));
        assert!(!is_global("GP_env_prod"));
        assert!(!is_global("P_env_prod"));

        assert!(!is_project("G_env_prod"));
        assert!(is_project("P_env_prod"));

        assert!(is_global_snapshot("GP_env_prod"));
        assert!(!is_global_snapshot("G_env_prod"));
    }

    #[test]
    fn test_roundtrip_snapshot() {
        let gid = "G_net_proxy";
        let sn = to_snapshot_id(gid).unwrap();
        assert_eq!(sn, "GP_net_proxy");
        let src = source_global_id(&sn).unwrap();
        assert_eq!(src, gid);
    }
}