// Version display module for DuckTape
//
// This module provides a direct way to display the version information
// by using the static version generated during build time.

include!("static_version.rs");

/// Returns the current application version from the static reference
pub fn get_version() -> &'static str {
    STATIC_VERSION
}

/// Displays the version information to stdout
pub fn display_version() {
    println!("DuckTape v{}", STATIC_VERSION);
    println!(
        "A tool for interacting with Apple Calendar, Notes, and Reminders via the command line."
    );
    println!("© 2024-2025 DuckTape Team");
}
