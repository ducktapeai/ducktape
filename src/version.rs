//! Version information for the DuckTape application.
//!
//! This module provides centralized access to version information,
//! ensuring consistent version reporting throughout the application.

// Include the generated version file from build.rs
include!(concat!(env!("OUT_DIR"), "/version.rs"));

/// Returns the current application version
pub fn get_version() -> &'static str {
    VERSION
}

/// Returns a formatted version string for display purposes
pub fn get_display_version() -> String {
    format!("v{}", VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_get_version_returns_version() {
        assert_eq!(get_version(), VERSION);
    }

    #[test]
    fn test_get_display_version_format() {
        assert_eq!(get_display_version(), format!("v{}", VERSION));
    }
}
