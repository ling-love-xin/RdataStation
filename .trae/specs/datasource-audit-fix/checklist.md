# 新增数据源功能 — 全链路 Checklist + 进度分析

> 最后更新：2026-06-09
> 状态：审计完成，16 项修复全部完成

---

## 一、修复总览

| 优先级 | 总数 | 已修复 | 修复内容 |
|:---:|:---:|:---:|------|
| 🔴 P0 | 2 | 2 | scope 双向选择 + saveToProject ID |
| 🟡 P1 | 6 | 6 | 字段一致性 + addStaging + NetworkTab + DuckDB + os_auth + driver_kind |
| 🟢 P2 | 8 | 8 | url_template + findIndex + localStorage + savingRef + configSchema + 后端校验 + 关闭确认 |

### 已修复问题清单

| # | 优先级 | 文件 | 问题 | 修复 |
|:---:|:---:|------|------|------|
| 1 | 🔴 | useAddDataSource.ts | scope 序列化丢失 'both' | 三态三元表达式 |
| 2 | 🔴 | AddDataSourceDialog.vue | saveToProject 用原始 ID 而非快照后 ID | 改用 snapshotNetId/snapshotAuthId |
| 3 | 🟡 | AddDataSourceDialog.vue | saveToStaging 从 formData 读高级字段 | 改用独立 ref |
| 4 | 🟡 | useAddDataSource.ts | addStaging 重置不完全 | 补充 formData/protocolChain/envId/driverProps |
| 5 | 🟡 | NetworkTab.vue | scope.project 变化不重载 | watch(props.scope?.project) |
| 6 | 🟡 | DuckDBAccelSection.vue | 所有驱动显示 DuckDB | isDuckDBSupported 过滤 |
| 7 | 🟡 | AddDataSourceDialog.vue | os_auth/trust 弹窗 | 已排除（isAuthRequired 正确） |
| 8 | 🟡 | GeneralTab.vue | driver_kind 未差异化 | getDefaultFields 已有 |
| 9 | 🟢 | useAddDataSource.ts | localStorage 写满静默 | message.warning 通知 |
| 10 | 🟢 | AddDataSourceDialog.vue | findIndex 脆弱 fallback | 仅用 UUID 匹配 |
| 11 | 🟢 | AddDataSourceDialog.vue | saving ref 测试按钮 | 已有 :loading="testing" |
| 12 | 🟢 | useAddDataSource.ts | url_template 空值替换 | 填入实际 formData 值 |
| 13 | 🟢 | GeneralTab.vue | config_schema 降级 | driver_kind 差异化降级 |
| 14 | 🟢 | connection_commands.rs | 后端 name/port/url 无校验 | 添加正向验证 |
| 15 | 🟢 | AddDataSourceDialog.vue | handleClose 无确认 | NModal 确认弹窗 |
| 16 | 🟢 | i18n zh-CN/en | 关闭确认无国际化 | 3 个新 key |

## 二、验证结果

| 检查项 | 结果 |
|------|:---:|
| `cargo check` | ✅ 零错误 |
| `pnpm run lint` (ESLint) | ✅ 零错误，exit 0 |
| `vue-tsc --noEmit` | ✅ 零新增错误（预存错误均在无关文件） |

## 三、修改文件清单（16 个文件）

```
src-tauri/src/core/services/connection_service.rs ✏️ 后端输入验证
src-tauri/src/commands/connection_commands.rs       ✏️ 同上
src-tauri/src/commands/project_store_commands.rs     ✏️ 同上

src/extensions/builtin/connection/ui/composables/
  useAddDataSource.ts                                ✏️ scope 'both' + addStaging + localStorage + url_template
  useNetworkChain.ts                                 — 未修改

src/extensions/builtin/connection/ui/components/
  AddDataSourceDialog.vue                            ✏️ saveToProject ID + 字段来源 + findIndex + 关闭确认
  tabs/GeneralTab.vue                                ✏️ config_schema driver_kind 降级
  tabs/NetworkTab.vue                                ✏️ scope.project watch 重载
  tabs/DuckDBAccelSection.vue                        ✏️ isDuckDBSupported 过滤
  tabs/AdvancedTab.vue                               ✏️ showDuckDB computed
  tabs/advanced/EnvironmentSection.vue               — 未修改
  tabs/advanced/PolicySections.vue                   — 未修改
  tabs/advanced/MetadataSection.vue                  — 未修改
  DataSourceHeader.vue                               — 未修改

src/shared/locales/
  zh-CN.json                                         ✏️ 3 个 i18n key
  en.json                                            ✏️ 3 个 i18n key
```