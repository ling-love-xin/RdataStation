# 分析资源管理器 — 开发进度文档

> 版本：v1.3
> 最后更新：2026-05-08
> 作者：RdataStation 团队

---

## 一、总览

| 阶段              | 状态      | 完成度 | 开始日期   | 完成日期   |
| ----------------- | --------- | ------ | ---------- | ---------- |
| Phase 1: 基础建设 | ✅ 已完成 | 100%   | 2026-04-20 | 2026-04-25 |
| Phase 2: 核心功能 | ✅ 已完成 | 100%   | 2026-04-25 | 2026-05-01 |
| Phase 3: 深度优化 | ✅ 已完成 | 100%   | 2026-05-02 | 2026-05-08 |
| Phase 4: 高级特性 | 🔲 计划中 | 0%     | —          | —          |

---

## 二、Phase 1: 基础建设（✅ 已完成）

### 2.1 数据库 Schema

| 任务                                   | 状态 | 说明                                |
| -------------------------------------- | ---- | ----------------------------------- |
| `analytics_resources` 表               | ✅   | 资源主表，含软删除、版本、配置 JSON |
| `analytics_folders` 表                 | ✅   | 文件夹树形结构，含作用域            |
| `analytics_resource_folder` 关联表     | ✅   | 多对多关联，支持播放列表模式        |
| `analytics_tags` 表                    | ✅   | 标签表，含作用域                    |
| `analytics_resource_tags` 关联表       | ✅   | 资源-标签多对多                     |
| `analytics_recycle_bin` 表             | ✅   | 回收站，含完整快照                  |
| 基础索引                               | ✅   | 类型、作用域、删除时间、名称        |
| 迁移文件 `007_analytics_resources.sql` | ✅   | v1.0 版本                           |

### 2.2 Rust 后端

| 任务                                   | 状态 | 说明                     |
| -------------------------------------- | ---- | ------------------------ |
| `AnalyticsResourceStore` 基础结构      | ✅   | ~800 行，7 表操作        |
| 资源 CRUD（create/read/update/delete） | ✅   | 软删除机制               |
| 文件夹 CRUD                            | ✅   | 树形结构支持             |
| 标签 CRUD + 关联操作                   | ✅   | 多对多关联               |
| 回收站（列表/恢复/永久删除）           | ✅   | 快照恢复机制             |
| 克隆资源                               | ✅   | 完整复制                 |
| `AnalyticsResourceState` 缓存          | ✅   | Arc<Mutex<Option<Store>> |
| Tauri Command 注册（lib.rs）           | ✅   | 18 个命令                |

### 2.3 前端基础

| 任务                                      | 状态 | 说明                   |
| ----------------------------------------- | ---- | ---------------------- |
| 类型定义 `types/index.ts`                 | ✅   | 完整 TS 类型体系       |
| API 层 `analytics-resource-api.ts`        | ✅   | tauri.invoke 封装      |
| Pinia Store `analytics-resource-store.ts` | ✅   | 状态管理核心           |
| `AnalyticsResourceManager.vue` 主组件     | ✅   | 事件协调中心           |
| `SearchBar.vue`                           | ✅   | 防抖搜索               |
| `FilterBar.vue`                           | ✅   | 作用域/类型筛选 + 排序 |
| `ResourceList.vue`                        | ✅   | 资源列表 + 单选/多选   |
| `Pagination.vue`                          | ✅   | 分页控件               |
| `CreateResourceModal.vue`                 | ✅   | 创建模态框             |

---

## 三、Phase 2: 核心功能（✅ 已完成）

### 3.1 文件夹管理

| 任务                     | 状态 | 说明                    |
| ------------------------ | ---- | ----------------------- |
| `FolderList.vue` 侧边栏  | ✅   | 文件夹列表展示          |
| 创建文件夹               | ✅   | `CreateFolderModal.vue` |
| 资源移动到文件夹         | ✅   | 多对多关联操作          |
| 资源从文件夹移除         | ✅   | 关联解除                |
| `resourceFolderMap` 映射 | ✅   | 资源→文件夹快速查找     |

### 3.2 回收站管理

| 任务                  | 状态 | 说明       |
| --------------------- | ---- | ---------- |
| `RecycleBinModal.vue` | ✅   | 回收站界面 |
| 恢复资源              | ✅   | 从快照恢复 |
| 永久删除              | ✅   | 不可逆删除 |

### 3.3 Context Menu

| 任务                | 状态 | 说明         |
| ------------------- | ---- | ------------ |
| `ContextMenu.vue`   | ✅   | 右键操作菜单 |
| 编辑/删除/复制/克隆 | ✅   | 四项核心操作 |

### 3.4 Toast 系统

| 任务                      | 状态 | 说明                         |
| ------------------------- | ---- | ---------------------------- |
| `ToastContainer.vue`      | ✅   | 消息提示容器                 |
| `use-toast.ts` composable | ✅   | 消息管理逻辑                 |
| 错误详情展开              | ✅   | `parseError()` + `ErrorInfo` |

### 3.5 Composables

| 任务                    | 状态 | 说明                |
| ----------------------- | ---- | ------------------- |
| `use-cache.ts` LRU 缓存 | ✅   | 前端 LRU + TTL 缓存 |
| `use-search.ts`         | ✅   | 搜索逻辑封装        |
| `use-debounce.ts`       | ✅   | 防抖工具            |

---

## 四、Phase 3: 深度优化（✅ 已完成）

### 4.1 安全性优化（P0）

| 任务           | 状态 | 说明                                                                      |
| -------------- | ---- | ------------------------------------------------------------------------- |
| 全部查询参数化 | ✅   | 替换字符串拼接为 `Vec<rusqlite::types::Value>`                            |
| SQL 注入防护   | ✅   | `list_resources_paginated`、`list_resources`、`list_folders`、`list_tags` |
| 统一参数传递   | ✅   | 所有方法统一使用参数化查询                                                |

### 4.2 数据完整性（P0）

| 任务                         | 状态 | 说明                                                |
| ---------------------------- | ---- | --------------------------------------------------- |
| 批量删除事务支持             | ✅   | `batch_delete_resources` 使用 BEGIN/COMMIT/ROLLBACK |
| 回收站恢复事务               | ✅   | `restore_from_recycle` 事务保护                     |
| 删除时关联表清理             | ✅   | 同步清理 `resource_folder` + `resource_tags`        |
| `parse_datetime_sqlite` 修复 | ✅   | 绕过 CoreError 转换，直接 parse                     |

### 4.3 Schema 增强（P1）

| 任务                                        | 状态 | 说明               |
| ------------------------------------------- | ---- | ------------------ |
| CHECK `json_valid(config)`                  | ✅   | JSON 格式校验      |
| CHECK `scope IN (...)`                      | ✅   | 作用域枚举约束     |
| `analytics_resource_versions` 表            | ✅   | 版本历史表         |
| 版本唯一约束 `UNIQUE(resource_id, version)` | ✅   | 防重复版本号       |
| `trg_ar_updated_at` 触发器                  | ✅   | 资源自动更新时间   |
| `trg_af_updated_at` 触发器                  | ✅   | 文件夹自动更新时间 |
| 标签唯一索引 `idx_at_name_scope`            | ✅   | 同名标签防重       |

### 4.4 标签双向查询（P1）

| 任务                        | 状态 | 说明                        |
| --------------------------- | ---- | --------------------------- |
| `get_tags_for_resource` API | ✅   | 查资源的标签列表            |
| `get_resources_by_tag` API  | ✅   | 查标签关联的资源            |
| Rust Store 实现             | ✅   | 参数化查询                  |
| Tauri Command 注册          | ✅   | 3 个新命令                  |
| 前端 API 封装               | ✅   | `analytics-resource-api.ts` |

### 4.5 版本历史（P1）

| 任务                             | 状态 | 说明                      |
| -------------------------------- | ---- | ------------------------- |
| `save_resource_version` 内部方法 | ✅   | 更新前自动保存旧版        |
| `get_resource_versions` API      | ✅   | 查询版本历史              |
| Rust Store 实现                  | ✅   | `ResourceVersion` 结构体  |
| 前端 API + 类型                  | ✅   | TS `ResourceVersion` 接口 |

### 4.6 设置面板（P2）

| 任务                                  | 状态 | 说明                          |
| ------------------------------------- | ---- | ----------------------------- |
| `SettingsModal.vue`                   | ✅   | 4 标签页面板                  |
| 通用设置（作用域/分页/排序）          | ✅   | 联动 store                    |
| 显示设置（图标/标签/元数据/虚拟滚动） | ✅   | 联动 ResourceList             |
| 缓存设置（开关/TTL/容量）             | ✅   | 联动 useCache                 |
| 快捷键参考页                          | ✅   | 全部快捷键列表                |
| `resetSettings()`                     | ✅   | 重置为默认                    |
| localStorage 持久化                   | ✅   | `analytics_resource_settings` |
| `applySettingsToState()` 联动         | ✅   | 保存立即生效                  |
| `updateConfig()` 动态缓存配置         | ✅   | 运行时调整                    |

### 4.7 快捷键全部实现

| 快捷键         | 功能         | 状态 |
| -------------- | ------------ | ---- |
| `Ctrl+N`       | 新建资源     | ✅   |
| `Ctrl+E`       | 编辑选中资源 | ✅   |
| `Ctrl+D`       | 删除选中资源 | ✅   |
| `Ctrl+Shift+C` | 克隆选中资源 | ✅   |
| `Ctrl+F`       | 聚焦搜索框   | ✅   |
| `Ctrl+A`       | 全选资源     | ✅   |
| `Delete`       | 删除选中资源 | ✅   |

### 4.8 标签管理 UI（🆕 v1.2）

| 任务                          | 状态 | 说明                                       |
| ----------------------------- | ---- | ------------------------------------------ |
| `TagManager.vue` 标签栏组件   | ✅   | 水平标签栏（全部标签 + 新建入口）          |
| `CreateTagModal.vue` 创建标签 | ✅   | 名称/颜色/作用域                           |
| 标签点击筛选资源              | ✅   | `getResourcesByTag` 按标签过滤             |
| 标签栏集成到主页面            | ✅   | FilterBar 下方                             |
| Store 方法封装                | ✅   | `getTagsForResource` / `getResourcesByTag` |

### 4.9 版本历史 UI（v1.2）

| 任务                      | 状态 | 说明                          |
| ------------------------- | ---- | ----------------------------- |
| `VersionHistoryModal.vue` | ✅   | 版本列表展示（JSON snapshot） |
| Store 方法封装            | ✅   | `getResourceVersions`         |
| 右键菜单入口              | ✅   | "查看版本历史" 选项           |
| `Ctrl+Shift+V` 快捷键     | ✅   | 选中资源后查看版本            |
| 设置面板快捷键更新        | ✅   | 快捷键列表新增                |

### 4.10 资源详情面板（🆕 v1.3）

| 任务                      | 状态 | 说明                                                          |
| ------------------------- | ---- | ------------------------------------------------------------- |
| `ResourceDetailModal.vue` | ✅   | 详情模态框（基本信息、统计、标签、文件夹、配置JSON、来源SQL） |
| 双击/打开资源触发         | ✅   | 替换 toast 占位为详情面板                                     |
| 标签动态加载              | ✅   | `getTagsForResource` 加载资源标签                             |
| 文件夹展示                | ✅   | 从 `resourceFolderMap` 读取                                   |
| 内置快捷操作              | ✅   | 编辑/复制/查看版本按钮                                        |

### 4.11 标签徽章展示（🆕 v1.3）

| 任务                   | 状态 | 说明                   |
| ---------------------- | ---- | ---------------------- |
| Store `resourceTagMap` | ✅   | 资源→标签批量加载映射  |
| `loadResourceTags()`   | ✅   | 批量加载当前页资源标签 |
| `getResourceTags()`    | ✅   | 获取单个资源的标签列表 |
| ResourceList 标签徽章  | ✅   | 每个资源项下方展示标签 |
| loadData 自动加载      | ✅   | 资源加载后自动获取标签 |

### 4.12 搜索历史（🆕 v1.3）

| 任务                          | 状态 | 说明                          |
| ----------------------------- | ---- | ----------------------------- |
| `useSearchHistory` composable | ✅   | localStorage 存储，最多 10 条 |
| SearchBar 历史下拉            | ✅   | 聚焦时展示，可点选/清除       |
| 自动记录                      | ✅   | Enter 搜索时自动保存          |

### 4.10 清理工作

| 任务                             | 状态 | 说明                             |
| -------------------------------- | ---- | -------------------------------- |
| 删除 `VirtualRoadmap.tsx` 死代码 | ✅   | 无引用，引入不存在的 VirtualGrid |
| 清理空目录                       | ✅   | 3 个空目录移除                   |
| 编辑模态框复用                   | ✅   | Create/Edit 双模式共享           |
| `OrderDetailModal` 类型修复      | ✅   | 添加 settled 状态                |
| `OrderListItem` 状态逻辑修复     | ✅   | 金额判断替代状态字符串           |

---

## 五、Phase 4: 高级特性（🔲 计划中）

### 5.1 待实现

| 功能                                   | 优先级 | 预估工时 | 依赖                     |
| -------------------------------------- | ------ | -------- | ------------------------ |
| 文件导入（CSV/Parquet/Excel → DuckDB） | 🔴 P0  | 3d       | DuckDB 引擎集成          |
| 从连接提取表到分析区                   | 🔴 P0  | 2d       | connection_manager       |
| SQL 查询结果自动转为资源               | 🔴 P0  | 2d       | query 模块集成           |
| 详情面板（右侧滑出）                   | 🟡 P1  | 2d       | dockview 面板 API        |
| 资源拖拽到 SQL 编辑器                  | 🟡 P1  | 2d       | dockview 拖拽 API        |
| 文件夹拖拽排序                         | 🟡 P1  | 1.5d     | 拖拽库                   |
| 资源批量导入                           | 🟡 P1  | 1d       | 文件选择器               |
| 依赖关系图                             | 🟡 P1  | 2d       | `resource_references` 表 |
| 删除前依赖检查                         | 🟡 P1  | 1d       | 依赖图                   |
| 操作审计日志                           | 🟢 P2  | 1d       | `audit_log` 表           |
| 资源模板                               | 🟢 P2  | 1.5d     | —                        |
| 智能推荐（最近/常用）                  | 🟢 P2  | 1d       | —                        |
| 批量提升为全局                         | 🟢 P2  | 1d       | —                        |
| 会话临时表自动清理                     | 🟢 P2  | 0.5d     | —                        |

### 5.2 技术债

| 项目                | 优先级 | 说明                                  |
| ------------------- | ------ | ------------------------------------- |
| Rust Store 拆分     | 🟡 P1  | ~1100 行单文件，考虑按职责拆分        |
| 前端虚拟滚动组件化  | 🟡 P1  | 从 ResourceList 抽离为独立 composable |
| 单元测试覆盖        | 🟡 P1  | Rust 和 TS 测试                       |
| WS 消息常量类型对齐 | 🟢 P2  | INT code vs string type 不一致        |

---

## 六、API 统计

| 模块     | 前端 API | Rust Command | Store 方法 | DB 表 |
| -------- | -------- | ------------ | ---------- | ----- |
| 资源     | 8        | 8            | 8          | 2     |
| 文件夹   | 5        | 5            | 5          | 2     |
| 标签     | 4        | 4            | 4          | 2     |
| 回收站   | 3        | 3            | 3          | 1     |
| 版本历史 | 1        | 1            | 2          | 1     |
| 双向查询 | 2        | 2            | 2          | —     |
| 初始化   | 1        | 1            | 1          | —     |
| **合计** | **24**   | **24**       | **25**     | **7** |

---

## 七、已知问题

| 编号 | 描述             | 优先级 | 状态 |
| ---- | ---------------- | ------ | ---- |
| —    | 无已知阻塞性 Bug | —      | ✅   |

---

## 八、版本历史

| 版本 | 日期       | 变更内容                                                                                                                              |
| ---- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| v1.3 | 2026-05-08 | 资源详情面板（ResourceDetailModal）、标签徽章展示（resourceTagMap + loadResourceTags）、搜索历史（useSearchHistory + SearchBar 下拉） |
| v1.2 | 2026-05-08 | 标签管理 UI（TagManager + CreateTagModal）、版本历史 UI（VersionHistoryModal + 右键菜单 + Ctrl+Shift+V）、Store 方法补齐（3 个）      |
| v1.1 | 2026-05-07 | P0/P1 深度优化：参数化查询、事务支持、Schema 增强、版本历史、标签双向查询、设置面板联动、快捷键全部实现                               |
| v1.0 | 2026-05-01 | Phase 1+2 完成：基础 CRUD、文件夹管理、回收站、Context Menu、Toast 系统、LRU 缓存                                                     |
| v0.1 | 2026-04-25 | Phase 1 完成：数据库 Schema、Rust Store 基础、前端框架搭建                                                                            |

---

## 九、文档清单

| 文档           | 路径                                                 | 说明               |
| -------------- | ---------------------------------------------------- | ------------------ |
| 架构设计       | `docs/frontend/ANALYTICS_RESOURCE_ARCHITECTURE.md`   | 完整架构文档       |
| 后端 Schema    | `docs/backend/ANALYTICS_RESOURCE_SCHEMA.md`          | 数据库设计与 API   |
| 设置文档       | `docs/frontend/ANALYTICS_RESOURCE_SETTINGS.md`       | 设置面板与联动机制 |
| 开发进度       | `docs/frontend/ANALYTICS_RESOURCE_PROGRESS.md`       | 本文档             |
| API 参考       | `docs/frontend/ANALYTICS_RESOURCE_API_REFERENCE.md`  | 接口详细参考       |
| 集成指南       | `docs/frontend/ANALYTICS_RESOURCE_INTEGRATION.md`    | 前端集成与使用指南 |
| 前端设计（旧） | `docs/frontend/ANALYTICS_RESOURCE_MANAGER_DESIGN.md` | v1.0 设计阶段文档  |
| 后端设计（旧） | `docs/backend/ANALYTICS_RESOURCE_MANAGER_DESIGN.md`  | v1.0 设计阶段文档  |
