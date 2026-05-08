use crate::core::mock::models::{
    ColumnDataType, ColumnDef, GeneratorConfig, ScenarioTemplate, TemplateTable,
};

macro_rules! col {
    ($name:expr, $data_type:expr, $gen:expr) => {
        ColumnDef {
            name: $name.to_string(),
            data_type: $data_type,
            generator: $gen,
            nullable_ratio: 0.0,
            unique: false,
        }
    };
    ($name:expr, $data_type:expr, $gen:expr, nullable) => {
        ColumnDef {
            name: $name.to_string(),
            data_type: $data_type,
            generator: $gen,
            nullable_ratio: 0.3,
            unique: false,
        }
    };
    ($name:expr, $data_type:expr, $gen:expr, unique) => {
        ColumnDef {
            name: $name.to_string(),
            data_type: $data_type,
            generator: $gen,
            nullable_ratio: 0.0,
            unique: true,
        }
    };
}

macro_rules! rnd_int {
    ($min:expr, $max:expr) => {
        GeneratorConfig::RandomInt { min: $min, max: $max }
    };
}

macro_rules! rnd_float {
    ($min:expr, $max:expr) => {
        GeneratorConfig::RandomFloat { min: $min, max: $max, precision: 2 }
    };
}

macro_rules! dt {
    ($min:expr, $max:expr) => {
        GeneratorConfig::DateTime { min: $min.to_string(), max: $max.to_string() }
    };
}

macro_rules! d {
    ($min:expr, $max:expr) => {
        GeneratorConfig::Date { min: $min.to_string(), max: $max.to_string() }
    };
}

fn ecommerce_template() -> ScenarioTemplate {
    ScenarioTemplate {
        id: "builtin:ecommerce".to_string(),
        name: "电商系统".to_string(),
        description: "包含用户、商品、订单、订单明细四张表，模拟典型 B2C 电商数据".to_string(),
        category: "商业".to_string(),
        locale: "zh_cn".to_string(),
        tables: vec![
            TemplateTable {
                name: "users".to_string(),
                row_count: 1000,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("username", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::Username, unique),
                    col!("email", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::SafeEmail, unique),
                    col!("phone", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::CellNumber),
                    col!("full_name", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::Name),
                    col!("birth_date", ColumnDataType::DateTime, d!("1960-01-01", "2005-12-31"), nullable),
                    col!("city", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::City),
                    col!("province", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::StateName),
                    col!("zip_code", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::ZipCode),
                    col!("registered_at", ColumnDataType::DateTime, dt!("2018-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                ],
            },
            TemplateTable {
                name: "products".to_string(),
                row_count: 500,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("sku", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::UuidV4, unique),
                    col!("name", ColumnDataType::Varchar { length: Some(200) }, GeneratorConfig::Words { min: 2, max: 5 }),
                    col!("category", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::Industry),
                    col!("description", ColumnDataType::Text, GeneratorConfig::Sentence { min: 5, max: 15 }, nullable),
                    col!("price", ColumnDataType::Float, rnd_float!(1.0, 99999.0)),
                    col!("cost", ColumnDataType::Float, rnd_float!(0.5, 50000.0)),
                    col!("stock_quantity", ColumnDataType::Integer, rnd_int!(0, 10000)),
                    col!("weight_grams", ColumnDataType::Integer, rnd_int!(10, 50000)),
                    col!("created_at", ColumnDataType::DateTime, dt!("2020-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                ],
            },
            TemplateTable {
                name: "orders".to_string(),
                row_count: 5000,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("order_number", ColumnDataType::Varchar { length: Some(30) }, GeneratorConfig::UuidV4, unique),
                    col!("user_id", ColumnDataType::Integer, rnd_int!(1, 1000)),
                    col!("status", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::Constant { value: "{\"options\":[\"pending\",\"processing\",\"shipped\",\"delivered\",\"cancelled\"]}".to_string() }),
                    col!("total_amount", ColumnDataType::Float, rnd_float!(10.0, 99999.0)),
                    col!("discount_amount", ColumnDataType::Float, rnd_float!(0.0, 5000.0)),
                    col!("payment_method", ColumnDataType::Varchar { length: Some(30) }, GeneratorConfig::CreditCardNumber),
                    col!("shipping_city", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::City),
                    col!("tracking_number", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::UuidV4, nullable),
                    col!("created_at", ColumnDataType::DateTime, dt!("2024-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                    col!("updated_at", ColumnDataType::DateTime, dt!("2024-01-01T00:00:00Z", "2025-12-31T23:59:59Z"), nullable),
                ],
            },
            TemplateTable {
                name: "order_items".to_string(),
                row_count: 15000,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("order_id", ColumnDataType::Integer, rnd_int!(1, 5000)),
                    col!("product_id", ColumnDataType::Integer, rnd_int!(1, 500)),
                    col!("quantity", ColumnDataType::Integer, rnd_int!(1, 10)),
                    col!("unit_price", ColumnDataType::Float, rnd_float!(1.0, 99999.0)),
                    col!("subtotal", ColumnDataType::Float, rnd_float!(1.0, 999999.0)),
                ],
            },
        ],
    }
}

fn hr_template() -> ScenarioTemplate {
    ScenarioTemplate {
        id: "builtin:hr".to_string(),
        name: "人力资源系统".to_string(),
        description: "包含员工、部门、薪资三张表，模拟企业内部 HR 管理数据".to_string(),
        category: "企业管理".to_string(),
        locale: "zh_cn".to_string(),
        tables: vec![
            TemplateTable {
                name: "employees".to_string(),
                row_count: 500,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("employee_code", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::UuidV4, unique),
                    col!("full_name", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::Name),
                    col!("english_name", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::FirstName, nullable),
                    col!("birth_date", ColumnDataType::DateTime, d!("1960-01-01", "2000-12-31")),
                    col!("phone", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::CellNumber),
                    col!("email", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::SafeEmail, unique),
                    col!("department_id", ColumnDataType::Integer, rnd_int!(1, 20)),
                    col!("position", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::Position),
                    col!("hire_date", ColumnDataType::DateTime, d!("2010-01-01", "2025-12-31")),
                    col!("salary_grade", ColumnDataType::Varchar { length: Some(10) }, GeneratorConfig::Constant { value: "P1,P2,P3,P4,P5,P6,M1,M2,M3".to_string() }),
                    col!("address", ColumnDataType::Text, GeneratorConfig::Sentence { min: 5, max: 15 }, nullable),
                ],
            },
            TemplateTable {
                name: "departments".to_string(),
                row_count: 20,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("name", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::CompanyName),
                    col!("manager_id", ColumnDataType::Integer, rnd_int!(1, 500), nullable),
                    col!("parent_id", ColumnDataType::Integer, rnd_int!(1, 10), nullable),
                    col!("budget", ColumnDataType::Float, rnd_float!(100000.0, 50000000.0)),
                    col!("headcount", ColumnDataType::Integer, rnd_int!(5, 200)),
                    col!("created_at", ColumnDataType::DateTime, d!("2010-01-01", "2025-12-31")),
                ],
            },
            TemplateTable {
                name: "salaries".to_string(),
                row_count: 500,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("employee_id", ColumnDataType::Integer, rnd_int!(1, 500)),
                    col!("year", ColumnDataType::Integer, rnd_int!(2022, 2025)),
                    col!("month", ColumnDataType::Integer, rnd_int!(1, 12)),
                    col!("base_salary", ColumnDataType::Float, rnd_float!(5000.0, 100000.0)),
                    col!("bonus", ColumnDataType::Float, rnd_float!(0.0, 50000.0), nullable),
                    col!("allowance", ColumnDataType::Float, rnd_float!(0.0, 10000.0)),
                    col!("deduction", ColumnDataType::Float, rnd_float!(0.0, 5000.0)),
                    col!("net_salary", ColumnDataType::Float, rnd_float!(4000.0, 150000.0)),
                    col!("paid_at", ColumnDataType::DateTime, dt!("2022-01-05T00:00:00Z", "2025-12-10T23:59:59Z"), nullable),
                ],
            },
        ],
    }
}

fn blog_template() -> ScenarioTemplate {
    ScenarioTemplate {
        id: "builtin:blog".to_string(),
        name: "博客 / 内容平台".to_string(),
        description: "包含文章、评论、标签三张表，模拟 UGC 内容平台数据".to_string(),
        category: "内容平台".to_string(),
        locale: "zh_cn".to_string(),
        tables: vec![
            TemplateTable {
                name: "articles".to_string(),
                row_count: 1000,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("title", ColumnDataType::Varchar { length: Some(200) }, GeneratorConfig::Sentence { min: 3, max: 10 }),
                    col!("slug", ColumnDataType::Varchar { length: Some(200) }, GeneratorConfig::UuidV4, unique),
                    col!("author_id", ColumnDataType::Integer, rnd_int!(1, 200)),
                    col!("content", ColumnDataType::Text, GeneratorConfig::Paragraph { count: 5 }),
                    col!("summary", ColumnDataType::Varchar { length: Some(500) }, GeneratorConfig::Sentence { min: 10, max: 30 }),
                    col!("category", ColumnDataType::Varchar { length: Some(100) }, GeneratorConfig::Industry),
                    col!("view_count", ColumnDataType::Integer, rnd_int!(0, 100000)),
                    col!("like_count", ColumnDataType::Integer, rnd_int!(0, 5000)),
                    col!("comment_count", ColumnDataType::Integer, rnd_int!(0, 500)),
                    col!("published_at", ColumnDataType::DateTime, dt!("2023-01-01T00:00:00Z", "2025-12-31T23:59:59Z"), nullable),
                    col!("created_at", ColumnDataType::DateTime, dt!("2022-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                    col!("updated_at", ColumnDataType::DateTime, dt!("2022-01-01T00:00:00Z", "2025-12-31T23:59:59Z"), nullable),
                ],
            },
            TemplateTable {
                name: "comments".to_string(),
                row_count: 5000,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("article_id", ColumnDataType::Integer, rnd_int!(1, 1000)),
                    col!("user_id", ColumnDataType::Integer, rnd_int!(1, 200)),
                    col!("parent_id", ColumnDataType::Integer, rnd_int!(1, 5000), nullable),
                    col!("content", ColumnDataType::Text, GeneratorConfig::Sentence { min: 3, max: 15 }),
                    col!("like_count", ColumnDataType::Integer, rnd_int!(0, 100)),
                    col!("created_at", ColumnDataType::DateTime, dt!("2023-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                ],
            },
            TemplateTable {
                name: "tags".to_string(),
                row_count: 100,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("name", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::Industry),
                    col!("slug", ColumnDataType::Varchar { length: Some(50) }, GeneratorConfig::UuidV4, unique),
                    col!("article_count", ColumnDataType::Integer, rnd_int!(1, 500)),
                    col!("created_at", ColumnDataType::DateTime, d!("2022-01-01", "2025-12-31")),
                ],
            },
        ],
    }
}

fn finance_template() -> ScenarioTemplate {
    ScenarioTemplate {
        id: "builtin:finance".to_string(),
        name: "金融 / 交易系统".to_string(),
        description: "包含交易记录、账户信息、产品三张表，模拟金融交易数据".to_string(),
        category: "金融".to_string(),
        locale: "en".to_string(),
        tables: vec![
            TemplateTable {
                name: "transactions".to_string(),
                row_count: 100000,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("txn_hash", ColumnDataType::Varchar { length: Some(64) }, GeneratorConfig::UuidV4, unique),
                    col!("account_id", ColumnDataType::Integer, rnd_int!(1, 500)),
                    col!("product_id", ColumnDataType::Integer, rnd_int!(1, 100)),
                    col!("amount", ColumnDataType::Float, rnd_float!(0.01, 10000000.0)),
                    col!("exchange_rate", ColumnDataType::Float, rnd_float!(0.01, 10.0)),
                    col!("fee", ColumnDataType::Float, rnd_float!(0.0, 1000.0)),
                    col!("description", ColumnDataType::Varchar { length: Some(500) }, GeneratorConfig::Sentence { min: 3, max: 10 }, nullable),
                    col!("ip_address", ColumnDataType::Varchar { length: Some(45) }, GeneratorConfig::IpAddress, nullable),
                    col!("executed_at", ColumnDataType::DateTime, dt!("2024-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                    col!("created_at", ColumnDataType::DateTime, dt!("2024-01-01T00:00:00Z", "2025-12-31T23:59:59Z")),
                ],
            },
            TemplateTable {
                name: "accounts".to_string(),
                row_count: 500,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("account_number", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::UuidV4, unique),
                    col!("account_name", ColumnDataType::Varchar { length: Some(200) }, GeneratorConfig::CompanyName),
                    col!("account_type", ColumnDataType::Varchar { length: Some(30) }, GeneratorConfig::Constant { value: "savings,checking,investment,credit,loan".to_string() }),
                    col!("balance", ColumnDataType::Float, rnd_float!(-1000000.0, 100000000.0)),
                    col!("currency", ColumnDataType::Varchar { length: Some(3) }, GeneratorConfig::CurrencyCode),
                    col!("interest_rate", ColumnDataType::Float, rnd_float!(0.0, 0.2)),
                    col!("opened_at", ColumnDataType::DateTime, d!("2015-01-01", "2025-12-31")),
                    col!("status", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::Constant { value: "active,dormant,frozen,closed".to_string() }),
                ],
            },
            TemplateTable {
                name: "products".to_string(),
                row_count: 100,
                columns: vec![
                    col!("id", ColumnDataType::Integer, GeneratorConfig::AutoIncrement { start: 1, step: 1 }, unique),
                    col!("isin_code", ColumnDataType::Varchar { length: Some(12) }, GeneratorConfig::Isin, unique),
                    col!("ticker", ColumnDataType::Varchar { length: Some(20) }, GeneratorConfig::UuidV4, unique),
                    col!("product_name", ColumnDataType::Varchar { length: Some(200) }, GeneratorConfig::CompanyName),
                    col!("product_type", ColumnDataType::Varchar { length: Some(30) }, GeneratorConfig::Constant { value: "stock,bond,fund,etf,option,future,crypto,forex".to_string() }),
                    col!("market_price", ColumnDataType::Float, rnd_float!(0.01, 100000.0)),
                    col!("nav", ColumnDataType::Float, rnd_float!(0.0, 100000.0)),
                    col!("market_cap", ColumnDataType::Float, rnd_float!(0.0, 1000000000000.0)),
                    col!("risk_level", ColumnDataType::Varchar { length: Some(10) }, GeneratorConfig::Constant { value: "low,medium,high,extreme".to_string() }),
                    col!("listed_date", ColumnDataType::DateTime, d!("2000-01-01", "2025-12-31")),
                ],
            },
        ],
    }
}

pub fn get_builtin_templates() -> Vec<ScenarioTemplate> {
    vec![
        ecommerce_template(),
        hr_template(),
        blog_template(),
        finance_template(),
    ]
}

pub fn get_template_by_id(id: &str) -> Option<ScenarioTemplate> {
    get_builtin_templates()
        .into_iter()
        .find(|t| t.id == id)
}