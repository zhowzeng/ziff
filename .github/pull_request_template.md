<!--
標題用 Conventional Commits：feat: / fix: / docs: / ci: / refactor: / test:
新名詞要對齊 CONTEXT.md；難反轉的決定要有 docs/decisions/ 的 ADR 背書。
用不到的段落直接刪掉。
-->

## 為什麼

<!-- 要解決的問題，一兩句。有 issue 就寫 Closes #N，這裡只補 issue 沒講到的部分。 -->

## 改了什麼

<!-- 依 reviewer 該讀的順序列，每項帶上檔案路徑。沒有行為改動就明講。 -->

## 沒有做的事

<!-- 刻意留在範圍外的東西與理由。有後續就寫 Refs #N。 -->

## 驗證

<!-- 實際跑過的指令與結果。沒跑的直接說沒跑，不要留白讓 reviewer 猜。 -->

- [ ] `bun run check`
- [ ] `bun run test`
- [ ] `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（動到 `src-tauri/` 才需要）
