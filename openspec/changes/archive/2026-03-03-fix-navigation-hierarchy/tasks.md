## 1. Fix navigate_back() Function

- [x] 1.1 Update `navigate_back()` for Spaces → Workspaces: repopulate sidebar with workspaces, restore selection using `current_workspace_id`
- [x] 1.2 Update `navigate_back()` for Folders → Spaces: repopulate sidebar with spaces, restore selection using `current_space_id`
- [x] 1.3 Update `navigate_back()` for Lists → Folders: repopulate sidebar with folders, restore selection using `current_folder_id`
- [x] 1.4 Update `navigate_back()` for Tasks → Lists: repopulate sidebar with lists, restore selection using `current_list_id`

## 2. Fix Async Message Handlers

- [x] 2.1 Update `AppMessage::WorkspacesLoaded` handler: after populating sidebar, restore selection using `current_workspace_id` if set, otherwise select first
- [x] 2.2 Update `AppMessage::SpacesLoaded` handler: after populating sidebar, restore selection using `current_space_id` if set, otherwise select first
- [x] 2.3 Update `AppMessage::FoldersLoaded` handler: after populating sidebar, restore selection using `current_folder_id` if set, otherwise select first
- [x] 2.4 Update `AppMessage::ListsLoaded` handler: after populating sidebar, restore selection using `current_list_id` if set, otherwise select first

## 3. Testing

- [x] 3.1 Test full navigation flow: Workspaces → Spaces → Folders → Lists → back to Workspaces
- [x] 3.2 Verify selection is restored at each level when navigating back
- [x] 3.3 Verify fallback behavior when tracked ID doesn't exist (item was deleted)
- [x] 3.4 Test session restore still works correctly after quitting and restarting
- [x] 3.5 Run `cargo test` to ensure no regressions

## 4. Cleanup

- [x] 4.1 Run `cargo clippy` and fix any warnings
- [x] 4.2 Run `cargo fmt` to ensure code is formatted
- [x] 4.3 Build release: `cargo build --release`
