# 新建数据库连接页面文档

## 概述

新建数据库连接页面是 RdataStation 的核心功能之一，用于创建和管理数据库连接。页面采用动态表单渲染架构，支持通过 JSON Schema 配置文件快速添加新的数据库类型，无需修改代码。

## 架构设计

### 目录结构

```
src/extensions/builtin/connection/ui/
├── components/
│   ├── ConnectionModal.vue          # 主模态框组件
│   ├── ConnectionSidebar.vue        # 左侧数据库类型树
│   ├── DynamicFormRenderer.vue      # 动态表单渲染器
│   ├── FieldRenderer.vue            # 字段渲染器
│   └── tabs/
│       ├── GeneralTab.vue           # 常规配置标签页
│       └── DuckdbAccelerationTab.vue # DuckDB 本地加速标签页
├── schemas/
│   ├── mysql.json                   # MySQL 连接配置
│   ├── postgresql.json              # PostgreSQL 连接配置
│   ├── sqlite.json                  # SQLite 连接配置
│   └── duckdb.json                  # DuckDB 连接配置
├── types/
│   └── form-schema.ts               # 表单类型定义
└── utils/
    └── schema-loader.ts             # Schema 加载器
```

### 核心组件

#### 1. ConnectionModal.vue

主模态框组件，包含以下功能：

- **头部区域**：标题、数据库图标、连接名称输入框、全局/项目多选
- **左侧边栏**：数据库类型树，支持搜索、最近使用
- **右侧内容区**：标签页导航和表单内容
- **底部操作栏**：测试连接、保存连接按钮

**标签页结构**：
- **常规**：动态渲染的数据库连接配置表单
- **本地加速**：DuckDB 本地加速配置（文件数据库置灰不可用）
- **驱动**：驱动信息和自定义选项

#### 2. DynamicFormRenderer.vue

动态表单渲染器，根据 JSON Schema 渲染表单：

- 支持多种字段类型：text、password、number、select、checkbox、file、textarea
- 支持字段依赖关系（dependsOn）
- 支持折叠区块（collapsible）
- 支持 inline 布局（紧凑排列）

#### 3. FieldRenderer.vue

字段渲染器组件，负责渲染单个表单字段：

- 处理不同类型的输入控件
- 处理错误状态显示
- 处理密码可见性切换
- 处理文件选择

## JSON Schema 配置

### Schema 结构

每个数据库类型的配置文件遵循以下结构：

```json
{
  "driverId": "mysql",
  "driverName": "MySQL",
  "version": "8.0",
  "metadata": {
    "category": "relational",
    "description": "MySQL 关系型数据库",
    "features": ["事务", "存储过程", "触发器", "视图"],
    "defaultPort": 3306,
    "requireFile": false,
    "supportsSsl": true,
    "supportsSshTunnel": true
  },
  "sections": [
    {
      "key": "connection",
      "title": "连接设置",
      "icon": "database",
      "collapsible": false,
      "fields": [...]
    }
  ]
}
```

### Metadata 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| category | string | 数据库类别：relational（网络数据库）、file（文件数据库） |
| description | string | 数据库描述 |
| features | string[] | 支持的功能特性 |
| defaultPort | number | 默认端口号 |
| requireFile | boolean | 是否需要文件路径（文件数据库为 true） |
| supportsSsl | boolean | 是否支持 SSL |
| supportsSshTunnel | boolean | 是否支持 SSH 隧道 |

### Section 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| key | string | 区块唯一标识 |
| title | string | 区块标题 |
| icon | string | 图标名称（lucide-vue-next） |
| collapsible | boolean | 是否可折叠 |
| collapsed | boolean | 默认是否折叠 |
| fields | FieldConfig[] | 字段配置数组 |

### Field 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| key | string | 字段唯一标识 |
| label | string | 字段标签 |
| type | string | 字段类型：text、password、number、select、checkbox、file、textarea |
| placeholder | string | 占位符文本 |
| required | boolean | 是否必填 |
| default | any | 默认值 |
| options | Option[] | 下拉选项（type 为 select 时使用） |
| validation | object | 验证规则 |
| tooltip | string | 提示文本 |
| hidden | boolean | 是否隐藏 |
| dependsOn | object | 依赖条件 |
| inline | boolean | 是否行内布局 |
| flex | number | 行内布局的 flex 比例 |

### 示例：添加新的数据库类型

要添加新的数据库类型（如 Oracle），只需创建 `oracle.json` 文件：

```json
{
  "driverId": "oracle",
  "driverName": "Oracle",
  "version": "21c",
  "metadata": {
    "category": "relational",
    "description": "Oracle 关系型数据库",
    "features": ["事务", "存储过程", "触发器", "视图", "分区表"],
    "defaultPort": 1521,
    "requireFile": false,
    "supportsSsl": true,
    "supportsSshTunnel": true
  },
  "sections": [
    {
      "key": "connection",
      "title": "连接设置",
      "icon": "database",
      "fields": [
        {
          "key": "host",
          "label": "主机",
          "type": "text",
          "placeholder": "localhost",
          "required": true,
          "inline": true,
          "flex": 2
        },
        {
          "key": "port",
          "label": "端口",
          "type": "number",
          "placeholder": "1521",
          "default": 1521,
          "required": true,
          "inline": true,
          "flex": 1
        },
        {
          "key": "serviceName",
          "label": "服务名",
          "type": "text",
          "placeholder": "ORCL",
          "required": true
        }
      ]
    },
    {
      "key": "authentication",
      "title": "认证",
      "icon": "user",
      "fields": [
        {
          "key": "username",
          "label": "用户名",
          "type": "text",
          "placeholder": "system",
          "required": true
        },
        {
          "key": "password",
          "label": "密码",
          "type": "password",
          "placeholder": "输入密码",
          "required": true
        }
      ]
    }
  ]
}
```

将文件放入 `schemas/` 目录后，页面会自动加载并渲染新的数据库类型配置表单。

## 布局设计

### DataGrip 风格布局

页面参考 DataGrip 的布局设计：

1. **左侧数据库类型树**：按类别分组显示所有支持的数据库类型
2. **右侧表单区域**：标签页导航 + 动态表单内容
3. **顶部连接名称**：直接在标题栏输入连接名称
4. **全局/项目多选**：允许同时保存到全局和项目

### 紧凑表单布局

- 主机和端口使用 inline 布局，按比例 2:1 排列
- 认证方式与认证信息在同一区块
- SSH/SSL 等高级选项默认折叠

### 文件数据库特殊处理

- 文件数据库（SQLite、DuckDB）不显示 SSH/SSL 配置
- 文件数据库的 DuckDB 加速标签页置灰不可用
- 文件数据库不显示内部的连接名称字段（已在标题栏）

## 功能特性

### 1. 动态表单渲染

- 基于 JSON Schema 配置
- 支持字段依赖关系
- 支持表单验证
- 支持默认值填充

### 2. 连接作用域

- **全局连接**：所有项目可用
- **项目连接**：仅当前项目可用
- 支持同时选择全局和项目

### 3. DuckDB 本地加速

- 仅对网络数据库可用（MySQL、PostgreSQL 等）
- 文件数据库的加速标签页置灰不可用
- 支持配置缓存策略、性能设置、过期策略

### 4. 最近使用

- 自动记录最近使用的 5 个数据库类型
- 在侧边栏顶部显示

### 5. 连接测试

- 点击"测试连接"按钮验证配置
- 显示测试结果

## 类型定义

### FormFieldConfig

```typescript
export interface FormFieldConfig {
  key: string
  label: string
  type: 'text' | 'password' | 'number' | 'select' | 'checkbox' | 'file' | 'textarea'
  placeholder?: string
  required?: boolean
  default?: unknown
  options?: Array<{ label: string; value: string | number }>
  validation?: {
    pattern?: string
    min?: number
    max?: number
    minLength?: number
    maxLength?: number
    message?: string
  }
  tooltip?: string
  hidden?: boolean
  dependsOn?: {
    field: string
    value: unknown
  }
  inline?: boolean
  flex?: number
}
```

### FormSectionConfig

```typescript
export interface FormSectionConfig {
  key: string
  title: string
  icon?: string
  description?: string
  fields: FormFieldConfig[]
  collapsible?: boolean
  collapsed?: boolean
}
```

### DriverFormSchema

```typescript
export interface DriverFormSchema {
  driverId: string
  driverName: string
  version: string
  metadata: {
    category: string
    description: string
    features: string[]
    defaultPort: number | null
    requireFile: boolean
    supportsSsl: boolean
    supportsSshTunnel: boolean
  }
  sections: FormSectionConfig[]
}
```

## 注意事项

1. **文件数据库**：`requireFile: true` 时，不显示主机、端口、SSH、SSL 等配置
2. **连接名称**：已在标题栏输入，表单内部不再需要
3. **DuckDB 加速**：仅对网络数据库可用，文件数据库置灰
4. **字段依赖**：使用 `dependsOn` 实现条件显示
5. **inline 布局**：使用 `inline: true` 和 `flex` 实现紧凑布局

## 扩展指南

### 添加新的数据库类型

1. 在 `schemas/` 目录创建 JSON 配置文件
2. 确保 `driverId` 与后端驱动标识一致
3. 配置 metadata 中的 `category` 和 `requireFile`
4. 定义 sections 和 fields

### 添加新的字段类型

1. 在 `FormFieldConfig` 类型中添加新类型
2. 在 `FieldRenderer.vue` 中实现渲染逻辑
3. 更新 CSS 样式

### 添加新的标签页

1. 在 `ConnectionModal.vue` 的 `visibleTabs` 中添加新标签页
2. 创建对应的组件
3. 在 `tabs-content` 区域渲染新组件
