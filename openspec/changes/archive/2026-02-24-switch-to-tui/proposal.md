## Why

The application currently uses the iced GUI framework which requires a graphical display server. Converting to a TUI (textual user interface) enables the application to run entirely within the terminal, improving accessibility for remote development environments, SSH sessions, and headless servers while reducing dependencies.

## What Changes

- **Replace iced GUI with Ratatui TUI framework** - The entire UI layer will be rewritten using Ratatui, a Rust TUI library built on crossterm
- **Remove windowing system dependencies** - No longer requires X11, Wayland, or other display servers
- **Discard screen ID system** - Remove the 4-character screen identifiers currently displayed in bottom-left corners
- **Implement screen titles** - Each screen will display a unique, descriptive title at the top for debugging and navigation clarity
- **Adapt navigation patterns** - Replace mouse-driven interactions with keyboard-first navigation
- **Redraw all UI components** - Sidebar, task lists, forms, and document views will be reimagined for terminal rendering

## Capabilities

### New Capabilities

- `tui-framework`: Core Ratatui integration including terminal setup, rendering loop, and crossterm event handling
- `tui-navigation`: Keyboard-based navigation system for workspace hierarchy (tabs, breadcrumbs, or menu-driven)
- `tui-forms`: Text input handling for authentication, task creation, and editing within terminal constraints
- `tui-layouts`: Screen layout system with unique titles displayed prominently on each screen
- `tui-widgets`: Custom terminal widgets for task lists with status/priority indicators, sidebar panels, and markdown preview

### Modified Capabilities

- `screen-identification`: **REMOVED** - The screen ID system is being replaced entirely by screen titles. The spec will be deleted.

## Impact

- **GUI Framework**: Remove iced, iced_widget, and related dependencies; add ratatui, crossterm
- **UI Layer**: Complete rewrite of all view modules (sidebar, task_list, task_detail, auth_view, document_view)
- **Event System**: Replace iced's Message pattern with crossterm event handling
- **Rendering**: Shift from immediate mode GUI to terminal buffer-based rendering
- **Input Handling**: Mouse support optional; primary interaction via keyboard
- **Screen Identification**: Remove screen ID generation and display logic; implement screen title system
- **Testing**: UI tests need updating for TUI rendering patterns
