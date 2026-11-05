# Tasks

- [x] 1. Extend `CommandMenuLevel` handling in `app.rs` with new commands (`t e/a/f/v`, group `c`, group `y`)
- [x] 2. Remove direct command keys (`n e d a s v r A u`) from `update_tasks` and `update_task_detail` normal-mode match arms
- [x] 3. Update `command_menu.rs` overlay text for groups `g/t/c/y/w`
- [x] 4. Update existing tests that send single-key commands to send SPACE sequences instead
- [x] 5. Add tests: direct `n`/`d` etc. in NORMAL mode are no-ops; SPACE sequences still fire
- [x] 6. `cargo test` + `cargo build` green
