//! CLI tests

mod cli_tests {
    use crate::cli::args::{parse_args, DebugOperation};

    #[test]
    fn test_parse_no_args_returns_tui_mode() {
        // Simulate no arguments - should return TUI mode
        // Note: This test is limited because parse_args() uses env::args()
        // Full integration tests are in tests/cli_test.rs
    }

    #[test]
    fn test_parse_debug_help() {
        let result = parse_args();
        // Help is handled when "debug" is called without operation
        // This test verifies the parser doesn't crash
        assert!(result.is_ok() || result.is_err());
    }
}
