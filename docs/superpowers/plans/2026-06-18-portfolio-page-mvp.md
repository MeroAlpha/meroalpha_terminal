# Portfolio Page MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first MeroAlpha Terminal desktop portfolio page with local holdings import, computed portfolio metrics, and a GPUI/gpui-component shell matching the design reference.

**Architecture:** Keep the portfolio math and CSV import independent from the GPUI view so it can be tested without a window. Store data behind a small repository boundary that can be backed by SQLite, while the first desktop page renders a snapshot of computed positions.

**Tech Stack:** Rust 2024, GPUI, gpui-component, SQLite via a repository module, and focused Rust unit tests for CSV/domain behavior.

---

### Task 1: Portfolio Domain And CSV Import

**Files:**
- Create: `src/lib.rs`
- Create: `src/portfolio.rs`

- [x] **Step 1: Write failing tests for CSV import and summary math**

Run: `cargo test --lib portfolio`

Expected before implementation: tests fail because `src/portfolio.rs` does not exist.

- [x] **Step 2: Implement portfolio domain**

Create `HoldingImport`, `PortfolioPosition`, `PortfolioSnapshot`, and `PortfolioImportError`.

- [x] **Step 3: Verify tests pass**

Run: `cargo test --lib portfolio`

Expected after implementation: all portfolio tests pass.

- [x] **Step 4: Support the observed MeroShare holdings export**

The importer accepts quoted CSV fields and maps `Scrip`, `Current Balance`, `Last Closing Price`, and `Last Transaction Price (LTP)` onto the internal holdings snapshot.

### Task 2: SQLite-Ready Storage Boundary

**Files:**
- Modify: `src/portfolio.rs`

- [x] **Step 1: Add a `PortfolioRepository` trait**

The trait stores imported holdings and loads the latest snapshot. The first pass includes both an in-memory implementation and a SQLite implementation backed by `rusqlite`.

- [x] **Step 2: Verify repository tests pass**

Run: `cargo test --lib portfolio`

Expected: repository tests pass.

- [x] **Step 3: Add SQLite repository coverage**

Use an in-memory SQLite database to prove imported holdings are persisted and loaded back into a computed portfolio snapshot.

### Task 3: Desktop App Shell And Portfolio Page

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/app.rs`

- [x] **Step 1: Add GPUI dependencies**

Use `gpui`, `gpui_platform`, `gpui-component`, and `gpui-component-assets` from their documented git sources.

- [x] **Step 2: Build the GPUI window shell**

Initialize `gpui_component::init(cx)`, wrap the app in `Root`, and render a dark shell with left navigation, top search, KPI cards, holdings ledger, and right insight rail.

- [x] **Step 3: Verify desktop build**

Run: `cargo check`

Expected: app compiles, or report external dependency/network blockers with exact error output.

### Task 4: Handoff Notes

**Files:**
- Modify: `README.md`

- [x] **Step 1: Document current portfolio CSV shape**

Supported columns are `symbol`, `quantity`, `avg_cost`, and `ltp`.

- [x] **Step 2: Document next slices**

Next slices are native file picker import, SQLite persistence, market-data sync, and broker analysis.
