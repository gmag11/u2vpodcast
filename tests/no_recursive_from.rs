//! Regression guard against recursive `From` impls (openspec: crash-safety).
//!
//! Some `From` implementations previously called `.into()` on their own
//! argument, recursing forever (stack overflow) on first use. This test scans
//! the crate's source and fails if any `impl From<...>` body contains a bare
//! `.into()` call, so the pattern can never be reintroduced silently.

use std::path::Path;

fn impl_from_blocks(source: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("impl From<") {
        // Expand from the start of `impl From<` until its closing brace,
        // counting braces so nested structs/unions are handled.
        let mark = rest[start..].find('{').map(|i| start + i);
        let Some(open) = mark else { break };
        let mut depth = 0usize;
        let mut end = None;
        for (offset, ch) in rest[open..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(open + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else { break };
        blocks.push(&rest[start..=end]);
        rest = &rest[end + 1..];
    }
    blocks
}

fn collect_rs_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn no_recursive_from_impl_bodies() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0usize;
    for file in collect_rs_files(&src_dir) {
        let source = std::fs::read_to_string(&file).expect("reading source file");
        for block in impl_from_blocks(&source) {
            checked += 1;
            assert!(
                !block.contains(".into()"),
                "recursive conversion found in {}:\n{}",
                file.display(),
                block
            );
        }
    }
    assert!(checked > 0, "no From impls found to check; guard is vacuous");
}