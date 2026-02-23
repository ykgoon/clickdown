## 1. Core Screen ID Module

- [x] 1.1 Create `src/screen_id.rs` module with ID generation logic (hash-based, 4-char alphanumeric)
- [x] 1.2 Add `generate_screen_id(screen_name: &str) -> String` function using deterministic hashing
- [x] 1.3 Add unit tests for ID generation (format validation, consistency, uniqueness)

## 2. UI Component for Screen ID Display

- [x] 2.1 Create `src/ui/screen_id_overlay.rs` with overlay rendering component
- [x] 2.2 Style the ID display (small monospace font, low contrast gray, bottom-left positioning)
- [x] 2.3 Export `ScreenIdOverlay` component for use in view functions

## 3. Integrate Screen IDs into All Views

- [x] 3.1 Add screen ID to authentication view (`src/ui/auth_view.rs`)
- [x] 3.2 Add screen ID to workspace/sidebar view (`src/ui/sidebar.rs`)
- [x] 3.3 Add screen ID to task list view (`src/ui/task_list.rs`)
- [x] 3.4 Add screen ID to task detail view (`src/ui/task_detail.rs`)
- [x] 3.5 Add screen ID to document view (`src/ui/document_view.rs`)
- [x] 3.6 Add screen ID to login view (`src/ui/login_view.rs`)

## 4. Module Integration and Testing

- [x] 4.1 Add `screen_id` module to `src/lib.rs` or `src/main.rs`
- [x] 4.2 Verify all screens display unique IDs when running the application
- [x] 4.3 Test ID consistency across application restarts
- [x] 4.4 Manual visual verification of ID positioning and styling
