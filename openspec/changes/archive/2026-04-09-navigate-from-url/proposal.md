## Why

ClickDown already supports copying resource URLs to the clipboard (via `Ctrl+Shift+C`/`Cmd+Shift+C`), letting users open resources in the web app. However, users who receive a ClickUp URL (e.g., from Slack, email, or a teammate) have no way to navigate directly to that resource within the TUI — they must manually browse to it. This change enables users to paste or type a ClickUp URL and have the application navigate to the corresponding resource, closing the loop on URL-based navigation.

## What Changes

- Add a URL input mode (triggered by a keyboard shortcut) that accepts a ClickUp resource URL
- Parse ClickUp URLs to extract workspace, space, folder, list, task, comment, and document identifiers
- Navigate the user to the corresponding resource in the TUI hierarchy, loading intermediate views as needed
- Display an error message if the URL is invalid, unrecognizable, or the resource cannot be loaded
- Add keyboard shortcut `g` then `u` (go-to URL) to trigger the URL input mode

## Capabilities

### New Capabilities
- `url-navigation`: Parse ClickUp URLs and navigate the TUI to the corresponding resource (workspace, space, folder, list, task, comment, or document)

### Modified Capabilities
- `keyboard-shortcuts`: Add the `g` then `u` chord shortcut to trigger URL input mode
- `help-dialog`: Document the new URL navigation shortcut in the help dialog

## Impact

- **`src/tui/app.rs`**: New URL input mode state handling and navigation logic
- **`src/tui/input.rs`**: New key chord (`g` → `u`) for triggering URL input
- **`src/tui/widgets/`**: New URL input widget (text input form for pasting/typing URLs)
- **URL parsing logic**: Reuse existing URL generation patterns from `element-url-copying` in reverse (parse URL path segments back into resource IDs)
- **Navigation system**: May require async loading of resources when navigating from a URL (e.g., navigating to a task URL may require loading the list first)
- **`openspec/specs/element-url-copying/`**: Reference for URL patterns to ensure parsing symmetry
