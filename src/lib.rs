use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use wax::walk::{Entry, FileIterator};

/// The same glob patterns used in the current main program.
pub const PATTERNS: &[&str] = &[
    "**/*.{c,cc,cxx,cpp,h,hpp,hxx}",
    "**/*.{cmake,cmake.in}",
    "**/CMakeFiles.txt",
    // "**/opencv/*",
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
            .not("**/.*/**")
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
    fn has_hidden_component(path: &Path) -> bool {
        use std::path::Component;
        path.components().any(|c| match c {
            Component::Normal(os) => {
                let s = os.to_string_lossy();
                s.starts_with('.') && s != "."
            }
            _ => false,
        })
    }

    // Build overrides to include our patterns.
    let mut ob = OverrideBuilder::new(root);
    // ob.add("**/.*/**")?;
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
        if has_hidden_component(dent.path()) {
            continue;
        }
        results.push(dent.path().to_path_buf());
    }
    Ok(results)
}

/// Collect matches using the `globwalk` crate, excluding all hidden entries
/// (any path component starting with a dot), e.g., `.pixi`, `.git`, `.env`.
pub fn collect_with_globwalk(
    root: &Path,
    patterns: &[&str],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut patterns = patterns.to_vec();
    // patterns.push("!**/.*/**");
    patterns.push("!.*");
    patterns.push("!.*/**");
    let walker = globwalk::GlobWalkerBuilder::from_patterns(root, &patterns)
        .follow_links(false)
        .build()?;

    // fn has_hidden_component(path: &Path) -> bool {
    //     use std::path::Component;
    //     path.components().any(|c| match c {
    //         Component::Normal(os) => {
    //             let s = os.to_string_lossy();
    //             s.starts_with('.') && s != "."
    //         }
    //         _ => false,
    //     })
    // }

    let mut results = Vec::new();
    for dent in walker.into_iter().filter_map(Result::ok) {
        // Skip directories; focus on file-like entries.
        if dent.file_type().is_dir() {
            continue;
        }
        let p = dent.path();
        // if has_hidden_component(p) {
        //     continue;
        // }
        results.push(p.to_path_buf());
    }
    Ok(results)
}

/// A simple BFS directory traversal using `globset` for include/exclude matching.
/// It prunes subtrees early when a directory matches any exclude glob.
pub fn collect_with_bfs_fastglob(
    root: &Path,
    include_patterns: &[&str],
    exclude_patterns: &[&str],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let to_match_str = |p: &Path| -> String {
        if let Ok(rel) = p.strip_prefix(&root) {
            rel.to_string_lossy().into_owned()
        } else {
            let s = p.to_string_lossy();
            if let Some(rest) = s.strip_prefix("./") {
                rest.to_string()
            } else {
                s.into_owned()
            }
        }
    };

    let matches_any = |patterns: &[&str], path: &Path, is_dir: bool| -> bool {
        let mut s = to_match_str(path);
        if is_dir && !s.ends_with('/') {
            s.push('/');
        }
        patterns
            .iter()
            .any(|pat| fast_glob::glob_match(pat.as_bytes(), s.as_bytes()))
    };

    let mut out = Vec::new();
    let mut q = VecDeque::new();
    q.push_back(root.to_path_buf());

    while let Some(dir) = q.pop_front() {
        // Prune subtree if excluded by any exclude glob.
        if matches_any(exclude_patterns, &dir, true) {
            continue;
        }

        let read_dir = match std::fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(_) => continue, // ignore unreadable directories
        };
        for ent in read_dir.filter_map(Result::ok) {
            let path = ent.path();
            match ent.file_type() {
                Ok(ft) if ft.is_dir() => {
                    if !matches_any(exclude_patterns, &path, true) {
                        q.push_back(path);
                    }
                }
                Ok(ft) if ft.is_file() => {
                    // Filter out hidden files regardless of excludes for consistent semantics.
                    if to_match_str(&path)
                        .split('/')
                        .any(|seg| seg.starts_with('.') && seg != ".")
                    {
                        continue;
                    }
                    if matches_any(include_patterns, &path, false)
                        && !matches_any(exclude_patterns, &path, false)
                    {
                        out.push(path);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(out)
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
            Component::Normal(os) => {
                os.to_string_lossy().starts_with('.') && os.to_string_lossy() != "."
            }
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

    #[test]
    fn globwalk_excludes_pixi() {
        let root = Path::new(".");
        let res = collect_with_globwalk(root, PATTERNS).expect("globwalk collection failed");
        assert!(
            res.iter().all(|p| !has_pixi_component(p)),
            "globwalk results contain .pixi entries: {:?}",
            res
        );
        // Sanity: ensure we found something under patterns so the test is meaningful in this repo
        // (avoid false positives if patterns matched nothing).
        assert!(res.len() > 0, "globwalk produced no results under patterns");
    }

    #[test]
    fn globwalk_excludes_all_hidden() {
        let root = Path::new(".");
        let res = collect_with_globwalk(root, PATTERNS).expect("globwalk collection failed");
        assert!(
            res.iter().all(|p| !has_hidden_component(p)),
            "globwalk results contain hidden entries: {:?}",
            res
        );
    }

    #[test]
    fn bfs_fastglob_excludes_pixi() {
        let root = Path::new(".");
        let res = collect_with_bfs_fastglob(root, PATTERNS, &["**/.*/**"])
            .expect("bfs collection failed");
        eprintln!("bfs_fastglob results: {} items", res.len());
        for p in res.iter().take(5) {
            eprintln!(" -> {}", p.display());
        }
        assert!(
            res.iter().all(|p| !has_pixi_component(p)),
            "bfs fastglob results contain .pixi entries: {:?}",
            res
        );
        assert!(
            res.len() > 0,
            "bfs fastglob produced no results under patterns"
        );
    }

    #[test]
    fn bfs_fastglob_excludes_all_hidden() {
        let root = Path::new(".");
        let res = collect_with_bfs_fastglob(root, PATTERNS, &["**/.*/**"])
            .expect("bfs collection failed");
        assert!(
            res.iter().all(|p| !has_hidden_component(p)),
            "bfs fastglob results contain hidden entries: {:?}",
            res
        );
    }

    #[test]
    fn fastglob_matching_smoke() {
        assert!(fast_glob::glob_match("**/opencv/*", "./opencv/LICENSE"));
        assert!(fast_glob::glob_match("**/opencv/*", "opencv/LICENSE"));
        assert!(fast_glob::glob_match("**/opencv/*", "a/b/opencv/LICENSE"));
        assert!(!fast_glob::glob_match("**/opencv/*", "a/b/opencv/sub/FILE"));
    }
}
