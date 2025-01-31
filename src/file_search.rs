use anyhow::Result;
use walkdir::WalkDir;

pub fn search(path: &str, pattern: &str) -> Result<()> {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let filename = entry.file_name().to_string_lossy();
        if filename.contains(pattern) {
            println!("{}", entry.path().display());
        }
    }
    Ok(())
}
