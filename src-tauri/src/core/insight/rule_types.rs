use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub category: String,
    pub applies_to: Vec<String>,
    #[serde(default)]
    pub builtin: bool,
}

fn default_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleFile {
    pub meta: RuleMeta,
    pub query: RuleQuery,
    #[serde(default)]
    pub output: Vec<OutputField>,
    #[serde(default)]
    pub quality: Option<Vec<QualityRule>>,
    #[serde(default)]
    pub render: Option<RenderHint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleQuery {
    pub template: String,
    #[serde(default)]
    pub parameters: Vec<String>,
    pub result_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputField {
    pub sql_name: String,
    pub json_name: String,
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRule {
    pub field: String,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderHint {
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub display_order: Option<i64>,
}
