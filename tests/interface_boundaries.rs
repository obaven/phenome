use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn assert_no_forbidden_dependencies(paths: &[PathBuf], forbidden: &[&str]) {
    for path in paths {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {}: {}", path.display(), err));
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for needle in forbidden {
                if trimmed.contains(needle) {
                    panic!(
                        "forbidden dependency '{}' found in {}:{}",
                        needle,
                        path.display(),
                        line_no + 1
                    );
                }
            }
        }
    }
}

#[test]
fn ui_core_has_no_terminal_or_ratatui_deps() {
    let roots = [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interfaces/ui_core"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/ui/rotappo-ui-core/src"),
    ];
    let mut files = Vec::new();
    for root in roots {
        if root.exists() {
            collect_rs_files(&root, &mut files)
                .unwrap_or_else(|err| panic!("failed to scan {}: {}", root.display(), err));
        }
    }
    assert!(
        !files.is_empty(),
        "no ui-core sources found in src/interfaces/ui_core or lib/ui/rotappo-ui-core/src"
    );

    let forbidden = [
        "ratatui",
        "crossterm",
        "interfaces::terminal",
        "interfaces::tui",
        "rotappo_ui_terminal",
        "rotappo_ui_tui",
    ];
    assert_no_forbidden_dependencies(&files, &forbidden);
}
