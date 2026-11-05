# Tasks

- [x] 1. Simplify `render_status` in `src/tui/layout.rs` to render only the status message (no hints, no border)
- [x] 2. Change status area constraint from `Length(3)` to `Length(1)` in `TuiLayout::new`
- [x] 3. Delete `get_hints()` from `src/tui/app.rs` and update the render call site
- [x] 4. Remove/update tests that assert hint text (`tests/tui_test.rs`, `tests/snapshot_test.rs`)
- [x] 5. `cargo test` + `cargo build` green; regenerate snapshots
