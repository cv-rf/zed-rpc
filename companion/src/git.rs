use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub repo_name: Option<String>,
    pub branch: Option<String>,
}

pub fn get_git_info(start_path: &Path) -> GitInfo {
    // 1. Try file-based detection first (fast, zero process overhead)
    if let Some(info) = detect_from_filesystem(start_path) {
        return info;
    }

    // 2. Fallback to git CLI if available
    detect_from_cli(start_path).unwrap_or_default()
}

fn detect_from_filesystem(start_path: &Path) -> Option<GitInfo> {
    let mut current = if start_path.is_file() {
        start_path.parent()?
    } else {
        start_path
    };

    loop {
        let git_candidate = current.join(".git");
        if git_candidate.exists() {
            let repo_name = current
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());

            let git_dir = if git_candidate.is_file() {
                // Worktree or submodule pointing to another git dir
                if let Ok(content) = fs::read_to_string(&git_candidate) {
                    if let Some(rest) = content.trim().strip_prefix("gitdir:") {
                        let path = rest.trim();
                        if Path::new(path).is_absolute() {
                            PathBuf::from(path)
                        } else {
                            current.join(path)
                        }
                    } else {
                        git_candidate
                    }
                } else {
                    git_candidate
                }
            } else {
                git_candidate
            };

            let head_path = git_dir.join("HEAD");
            let branch = if let Ok(head_content) = fs::read_to_string(&head_path) {
                let trimmed = head_content.trim();
                if let Some(branch_name) = trimmed.strip_prefix("ref: refs/heads/") {
                    Some(branch_name.to_string())
                } else if trimmed.len() >= 7 {
                    Some(trimmed[..7].to_string())
                } else {
                    None
                }
            } else {
                None
            };

            return Some(GitInfo { repo_name, branch });
        }

        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => break,
        }
    }

    None
}

fn detect_from_cli(start_path: &Path) -> Option<GitInfo> {
    let dir = if start_path.is_file() {
        start_path.parent()?
    } else {
        start_path
    };

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();

    let repo_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()?;

    let repo_name = if repo_output.status.success() {
        let top = String::from_utf8(repo_output.stdout).ok()?;
        Path::new(top.trim())
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    Some(GitInfo {
        repo_name,
        branch: if branch.is_empty() { None } else { Some(branch) },
    })
}
