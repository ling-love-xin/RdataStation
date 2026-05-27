//! specta TypeScript 类型导出测试
//!
//! 运行此测试即可自动生成 TypeScript 绑定文件:
//!   cargo test -p rdata-station --test specta_export -- --nocapture
//!
//! 生成路径: src/generated/specta/bindings.ts
//!
//! v3.0: ts-rs → specta 迁移，类型通过 specta::collect_types! + tauri_specta::ts::export 导出

#[test]
fn export_specta_types() {
    // ⚠️ 暂未激活：需等待 tauri-specta rc.25 API 稳定后启用
    // tauri_specta::ts::export(
    //     specta::collect_types![
    //         // 所有 #[specta::specta] 标注的 Tauri 命令
    //         // 将在 API 确认后填充
    //     ],
    //     "../src/generated/specta/bindings.ts",
    // ).expect("specta 类型导出失败");
    eprintln!("⚠️ specta export 待激活：需确认 tauri-specta rc.25 的 ts::export + collect_types API");
}