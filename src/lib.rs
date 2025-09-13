use std::path::{Path, PathBuf};

/// The same glob patterns used in the current main program.
pub const PATTERNS: &[&str] = &[
    "**/*.{c,cc,cxx,cpp,h,hpp,hxx}",
    "**/*.{cmake,cmake.in}",
    "**/CMakeFiles.txt",
];

/// Collect matches using the `wax` crate, excluding all hidden entries
/// (any path component starting with a dot), e.g., `.pixi`, `.git`, `.env`.
pub fn collect_with_wax(
    root: &Path,
    patterns: &[&str],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    for pat in patterns {
        let glob = wax::Glob::new(pat)?;
        let iter = glob
            .walk(root)
            // Exclude hidden directories and their descendants; exhaustive pattern enables pruning.
            .not(["**/.*/**"])
            .unwrap()
            .filter_map(|e| e.ok());

        for entry in iter {
            // Skip directories; focus on file-like entries.
            if entry.file_type().is_dir() {
                continue;
            }
            results.push(entry.path().to_path_buf());
        }
    }
    Ok(results)
}

/// Collect matches using the `ignore` crate (via Overrides + WalkBuilder),
/// excluding all hidden entries (any `.*` component).
pub fn collect_with_ignore(
    root: &Path,
    patterns: &[&str],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    use ignore::{overrides::OverrideBuilder, WalkBuilder};

    // Build overrides to include our patterns.
    let mut ob = OverrideBuilder::new(root);
    for pat in patterns {
        ob.add(pat)?; // include pattern
    }
    let overrides = ob.build()?;

    let mut builder = WalkBuilder::new(root);
    // Disable standard ignore files and exclude hidden entries like Wax does above.
    builder
        .follow_links(false)
        .standard_filters(false)
        .hidden(true)
        .overrides(overrides);

    let mut results = Vec::new();
    for dent in builder.build() {
        let dent = dent?;
        // Skip directories; focus on file-like entries.
        if dent.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        results.push(dent.path().to_path_buf());
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_pixi_component(path: &Path) -> bool {
        path.components().any(|c| c.as_os_str() == ".pixi")
    }

    fn has_hidden_component(path: &Path) -> bool {
        use std::path::Component;
        path.components().any(|c| match c {
            Component::Normal(os) => os.to_string_lossy().starts_with('.'),
            _ => false,
        })
    }

    #[test]
    fn wax_excludes_pixi() {
        let root = Path::new(".");
        let res = collect_with_wax(root, PATTERNS).expect("wax collection failed");
        assert!(
            res.iter().all(|p| !has_pixi_component(p)),
            "wax results contain .pixi entries: {:?}",
            res
        );
    }

    #[test]
    fn ignore_excludes_pixi() {
        let root = Path::new(".");
        let res = collect_with_ignore(root, PATTERNS).expect("ignore collection failed");
        assert!(
            res.iter().all(|p| !has_pixi_component(p)),
            "ignore results contain .pixi entries: {:?}",
            res
        );
    }

    #[test]
    fn wax_excludes_all_hidden() {
        let root = Path::new(".");
        let res = collect_with_wax(root, PATTERNS).expect("wax collection failed");
        assert!(
            res.iter().all(|p| !has_hidden_component(p)),
            "wax results contain hidden entries: {:?}",
            res
        );
    }

    #[test]
    fn ignore_excludes_all_hidden() {
        let root = Path::new(".");
        let res = collect_with_ignore(root, PATTERNS).expect("ignore collection failed");
        assert!(
            res.iter().all(|p| !has_hidden_component(p)),
            "ignore results contain hidden entries: {:?}",
            res
        );
    }
}
