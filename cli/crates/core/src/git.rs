//! Git integration for axel workspaces.
//!
//! This module provides git worktree management, allowing axel to create
//! isolated working directories for different branches.
//!
//! # Worktree Workflow
//!
//! ```bash
//! axel -w feat/auth    # Create worktree + launch workspace
//! axel -w feat/auth -k # Kill workspace + optionally prune worktree
//! ```
//!
//! Worktrees are created as siblings to the main repository:
//! ```text
//! ~/code/myproject/              # main repo
//! ~/code/myproject-feat-auth/    # worktree for feat/auth
//! ~/code/myproject-fix-bug/      # worktree for fix/bug
//! ```

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

/// Result of ensuring a worktree exists.
#[derive(Debug)]
pub struct WorktreeInfo {
    /// Path to the worktree directory
    pub path: PathBuf,
    /// Branch name
    pub branch: String,
    /// Whether the worktree was newly created
    pub created: bool,
    /// Whether the branch was newly created
    pub branch_created: bool,
}

/// Check if we're inside a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the repository root directory.
pub fn repo_root(path: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .context("Failed to execute git")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(root))
}

/// Get the repository name (directory name of the repo root).
pub fn repo_name(path: &Path) -> Result<String> {
    let root = repo_root(path)?;
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .context("Could not determine repository name")
}

/// Check if a branch exists locally.
pub fn branch_exists_local(path: &Path, branch: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", branch),
        ])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a branch exists on a remote.
pub fn branch_exists_remote(path: &Path, branch: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "-r", "--list", &format!("*/{}", branch)])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        let remotes = String::from_utf8_lossy(&output.stdout);
        let remote = remotes.lines().next()?.trim();
        if !remote.is_empty() {
            return Some(remote.to_string());
        }
    }
    None
}

/// Get the current branch name.
pub fn current_branch(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        bail!("Failed to get current branch");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get the default branch (main or master).
pub fn default_branch(path: &Path) -> Result<String> {
    // Try to get from remote HEAD
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD", "--short"])
        .current_dir(path)
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Strip "origin/" prefix
        if let Some(name) = branch.strip_prefix("origin/") {
            return Ok(name.to_string());
        }
    }

    // Fallback: check if main or master exists
    if branch_exists_local(path, "main") {
        return Ok("main".to_string());
    }
    if branch_exists_local(path, "master") {
        return Ok("master".to_string());
    }

    // Last resort: use current branch
    current_branch(path)
}

/// List all worktrees for a repository.
pub fn list_worktrees(path: &Path) -> Result<Vec<(PathBuf, String)>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(path)
        .output()
        .context("Failed to list worktrees")?;

    if !output.status.success() {
        bail!("Failed to list worktrees");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;

    for line in stdout.lines() {
        if let Some(path_str) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path_str));
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/")
            && let Some(path) = current_path.take()
        {
            worktrees.push((path, branch.to_string()));
        }
    }

    Ok(worktrees)
}

/// Find existing worktree for a branch.
pub fn find_worktree(path: &Path, branch: &str) -> Result<Option<PathBuf>> {
    let worktrees = list_worktrees(path)?;
    Ok(worktrees
        .into_iter()
        .find(|(_, b)| b == branch)
        .map(|(p, _)| p))
}

/// Convert branch name to a valid directory name.
fn branch_to_dirname(branch: &str) -> String {
    branch.replace(['/', '\\'], "-")
}

/// Ensure a worktree exists for a branch, creating if necessary.
///
/// If the branch doesn't exist, it will be created from the default branch.
/// The worktree is created as a sibling directory to the repository.
pub fn ensure_worktree(path: &Path, branch: &str) -> Result<WorktreeInfo> {
    let repo_root = repo_root(path)?;
    let repo_name = repo_name(path)?;

    // Check if worktree already exists for this branch
    if let Some(existing_path) = find_worktree(path, branch)? {
        // Verify the worktree directory actually exists
        if existing_path.exists() {
            return Ok(WorktreeInfo {
                path: existing_path,
                branch: branch.to_string(),
                created: false,
                branch_created: false,
            });
        } else {
            // Worktree reference exists but directory is gone - prune stale references
            prune_worktrees(path)?;
        }
    }

    // Determine worktree path (sibling to repo)
    let worktree_name = format!("{}-{}", repo_name, branch_to_dirname(branch));
    let worktree_path = repo_root
        .parent()
        .context("Repository has no parent directory")?
        .join(&worktree_name);

    // Check if branch exists
    let branch_exists = branch_exists_local(path, branch);
    let remote_branch = branch_exists_remote(path, branch);
    let branch_created;

    if branch_exists {
        // Branch exists locally, create worktree
        branch_created = false;
        let status = Command::new("git")
            .args(["worktree", "add", worktree_path.to_str().unwrap(), branch])
            .current_dir(&repo_root)
            .status()
            .context("Failed to create worktree")?;

        if !status.success() {
            bail!("Failed to create worktree for branch '{}'", branch);
        }
    } else if let Some(remote) = remote_branch {
        // Branch exists on remote, track it
        branch_created = false;
        let status = Command::new("git")
            .args([
                "worktree",
                "add",
                "--track",
                "-b",
                branch,
                worktree_path.to_str().unwrap(),
                &remote,
            ])
            .current_dir(&repo_root)
            .status()
            .context("Failed to create worktree")?;

        if !status.success() {
            bail!(
                "Failed to create worktree tracking remote branch '{}'",
                remote
            );
        }
    } else {
        // Branch doesn't exist, create it from default branch
        branch_created = true;
        let base = default_branch(path)?;
        let status = Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                branch,
                worktree_path.to_str().unwrap(),
                &base,
            ])
            .current_dir(&repo_root)
            .status()
            .context("Failed to create worktree")?;

        if !status.success() {
            bail!(
                "Failed to create worktree with new branch '{}' from '{}'",
                branch,
                base
            );
        }
    }

    // Symlink AXEL.md if it exists in main repo but not in worktree
    let main_manifest = repo_root.join("AXEL.md");
    let worktree_manifest = worktree_path.join("AXEL.md");
    if main_manifest.exists() && !worktree_manifest.exists() {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&main_manifest, &worktree_manifest).ok();
        }
    }

    Ok(WorktreeInfo {
        path: worktree_path,
        branch: branch.to_string(),
        created: true,
        branch_created,
    })
}

/// Remove a worktree.
///
/// If `force` is true, removes even if there are uncommitted changes.
pub fn remove_worktree(path: &Path, branch: &str, force: bool) -> Result<bool> {
    let worktree_path = match find_worktree(path, branch)? {
        Some(p) => p,
        None => return Ok(false),
    };

    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree_path.to_str().unwrap());

    let status = Command::new("git")
        .args(&args)
        .current_dir(path)
        .status()
        .context("Failed to remove worktree")?;

    Ok(status.success())
}

/// Prune stale worktree references.
pub fn prune_worktrees(path: &Path) -> Result<()> {
    Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(path)
        .status()
        .context("Failed to prune worktrees")?;
    Ok(())
}

/// Result of completing an isolated worktree (squash + cherry-pick).
#[derive(Debug)]
pub struct CompleteResult {
    /// The commit hash of the squashed commit in the parent worktree
    pub commit_hash: String,
    /// Files changed in the squashed commit
    pub files_changed: Vec<String>,
    /// Number of insertions
    pub insertions: u32,
    /// Number of deletions
    pub deletions: u32,
}

/// Create an isolated (temporary) worktree from a source path.
///
/// Forks from the current HEAD of `source_path` (which may be a repo root or
/// existing worktree). By default, the source must have a clean working tree.
/// Pass `allow_dirty: true` to copy dirty/untracked files into the new worktree.
///
/// Returns `WorktreeInfo` with `branch` set to the temp branch name.
pub fn create_isolated_worktree(
    source_path: &Path,
    source_branch: &str,
    pane_id: &str,
    allow_dirty: bool,
) -> Result<WorktreeInfo> {
    let root = repo_root(source_path)?;
    let name = repo_name(source_path)?;
    let short_id = &pane_id[..8.min(pane_id.len())];
    let temp_branch = format!("axel-tmp/{}", short_id);

    // Check for dirty state
    let dirty_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(source_path)
        .output()
        .context("Failed to check git status")?;
    let dirty_files_str = String::from_utf8_lossy(&dirty_output.stdout).to_string();
    let has_dirty = !dirty_files_str.trim().is_empty();

    if has_dirty && !allow_dirty {
        bail!(
            "Source has uncommitted changes. Use --allow-dirty to include them in the isolated worktree."
        );
    }

    // Determine worktree path (sibling to repo root)
    let worktree_dir = format!("{}-axel-tmp-{}", name, short_id);
    let worktree_path = root
        .parent()
        .context("Repository has no parent directory")?
        .join(&worktree_dir);

    // Remove existing temp branch if it exists (leftover from previous run)
    if branch_exists_local(source_path, &temp_branch) {
        // Try to remove any existing worktree for this branch first
        let _ = remove_worktree(source_path, &temp_branch, true);
        Command::new("git")
            .args(["branch", "-D", &temp_branch])
            .current_dir(source_path)
            .output()
            .ok();
    }

    // Remove existing worktree directory if it exists
    if worktree_path.exists() {
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(source_path)
            .output()
            .ok();
        // If git worktree remove didn't clean it up, force remove
        if worktree_path.exists() {
            std::fs::remove_dir_all(&worktree_path).ok();
            prune_worktrees(source_path)?;
        }
    }

    // Create worktree from source branch
    let status = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            &temp_branch,
            worktree_path.to_str().unwrap(),
            source_branch,
        ])
        .current_dir(source_path)
        .status()
        .context("Failed to create isolated worktree")?;

    if !status.success() {
        bail!(
            "Failed to create isolated worktree '{}' from '{}'",
            temp_branch,
            source_branch
        );
    }

    // If --allow-dirty and source had dirty state, copy dirty files into the new worktree
    if has_dirty && allow_dirty {
        for line in dirty_files_str.lines() {
            let line = line.trim();
            if line.len() < 4 {
                continue;
            }
            let file_part = &line[3..];
            let file_path = if let Some(arrow_idx) = file_part.find(" -> ") {
                &file_part[arrow_idx + 4..]
            } else {
                file_part
            };

            let src = Path::new(source_path).join(file_path);
            let dst = worktree_path.join(file_path);

            let status_char = line.chars().next().unwrap_or(' ');
            let index_char = line.chars().nth(1).unwrap_or(' ');
            if status_char == 'D' || index_char == 'D' {
                if dst.exists() {
                    std::fs::remove_file(&dst).ok();
                }
                continue;
            }

            if src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::copy(&src, &dst).ok();
            }
        }
    }

    // Symlink AXEL.md if it exists in main repo but not in worktree
    let main_manifest = root.join("AXEL.md");
    let worktree_manifest = worktree_path.join("AXEL.md");
    if main_manifest.exists() && !worktree_manifest.exists() {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&main_manifest, &worktree_manifest).ok();
        }
    }

    Ok(WorktreeInfo {
        path: worktree_path,
        branch: temp_branch,
        created: true,
        branch_created: true,
    })
}

/// Squash all commits in a temp worktree since it diverged from parent,
/// cherry-pick the result to the parent worktree.
///
/// If `keep_worktree` is true, the temp worktree is reset to match the parent's
/// new HEAD instead of being removed. This allows the same Claude session to
/// continue working in the same directory for the next task.
pub fn complete_isolated_worktree(
    temp_worktree_path: &Path,
    parent_worktree_path: &Path,
    commit_message: &str,
    commit_description: &str,
    keep_worktree: bool,
) -> Result<CompleteResult> {
    // Get the temp branch name
    let temp_branch = current_branch(temp_worktree_path)?;

    // Get the parent branch name
    let parent_branch = current_branch(parent_worktree_path)?;

    // Find the merge base
    let merge_base_output = Command::new("git")
        .args(["merge-base", &parent_branch, &temp_branch])
        .current_dir(temp_worktree_path)
        .output()
        .context("Failed to find merge base")?;

    if !merge_base_output.status.success() {
        bail!(
            "Failed to find merge base between '{}' and '{}'",
            parent_branch,
            temp_branch
        );
    }
    let merge_base = String::from_utf8_lossy(&merge_base_output.stdout)
        .trim()
        .to_string();

    // Check if there are any changes to squash
    let diff_check = Command::new("git")
        .args(["diff", "--stat", &merge_base, "HEAD"])
        .current_dir(temp_worktree_path)
        .output()
        .context("Failed to check diff")?;

    let diff_stat = String::from_utf8_lossy(&diff_check.stdout)
        .trim()
        .to_string();

    if diff_stat.is_empty() {
        bail!("No changes to squash in isolated worktree");
    }

    // Build full commit message
    let full_message = if commit_description.is_empty() {
        commit_message.to_string()
    } else {
        format!("{}\n\n{}", commit_message, commit_description)
    };

    // Soft reset to merge base to squash all changes
    let status = Command::new("git")
        .args(["reset", "--soft", &merge_base])
        .current_dir(temp_worktree_path)
        .status()
        .context("Failed to soft reset")?;

    if !status.success() {
        bail!("Failed to soft reset to merge base");
    }

    // Create the squash commit
    let status = Command::new("git")
        .args(["commit", "-m", &full_message])
        .current_dir(temp_worktree_path)
        .status()
        .context("Failed to create squash commit")?;

    if !status.success() {
        bail!("Failed to create squash commit");
    }

    // Get the squash commit hash
    let hash_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp_worktree_path)
        .output()
        .context("Failed to get squash commit hash")?;

    let squash_hash = String::from_utf8_lossy(&hash_output.stdout)
        .trim()
        .to_string();

    // Cherry-pick the squash commit into the parent worktree
    let status = Command::new("git")
        .args(["cherry-pick", &squash_hash])
        .current_dir(parent_worktree_path)
        .status()
        .context("Failed to cherry-pick squash commit")?;

    if !status.success() {
        bail!(
            "Failed to cherry-pick squash commit {} to parent worktree. \
             There may be conflicts that need manual resolution.",
            &squash_hash[..8.min(squash_hash.len())]
        );
    }

    // Get the final commit hash in parent
    let final_hash_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(parent_worktree_path)
        .output()
        .context("Failed to get final commit hash")?;

    let final_hash = String::from_utf8_lossy(&final_hash_output.stdout)
        .trim()
        .to_string();

    // Get diff stats for the result
    let stat_output = Command::new("git")
        .args(["diff", "--stat", &format!("{}~1", final_hash), &final_hash])
        .current_dir(parent_worktree_path)
        .output()
        .context("Failed to get diff stats")?;

    let stat_text = String::from_utf8_lossy(&stat_output.stdout);
    let mut files_changed = Vec::new();
    let mut insertions = 0u32;
    let mut deletions = 0u32;

    for line in stat_text.lines() {
        let line = line.trim();
        if line.contains('|') {
            // File line: " path/to/file | N ++++---"
            if let Some(path) = line.split('|').next() {
                files_changed.push(path.trim().to_string());
            }
        } else if line.contains("insertion") || line.contains("deletion") {
            // Summary line: " N files changed, N insertions(+), N deletions(-)"
            for part in line.split(',') {
                let part = part.trim();
                if part.contains("insertion") {
                    if let Some(n) = part.split_whitespace().next() {
                        insertions = n.parse().unwrap_or(0);
                    }
                } else if part.contains("deletion") {
                    if let Some(n) = part.split_whitespace().next() {
                        deletions = n.parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    if keep_worktree {
        // Reset the temp worktree to match the parent's new HEAD.
        // This keeps the directory alive so the Claude session can continue
        // working in the same cwd for the next task.
        let parent_head = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(parent_worktree_path)
            .output()
            .context("Failed to get parent HEAD")?;
        let parent_hash = String::from_utf8_lossy(&parent_head.stdout)
            .trim()
            .to_string();

        Command::new("git")
            .args(["reset", "--hard", &parent_hash])
            .current_dir(temp_worktree_path)
            .status()
            .ok();
    } else {
        // Clean up: remove temp worktree and branch
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                temp_worktree_path.to_str().unwrap(),
            ])
            .current_dir(parent_worktree_path)
            .status()
            .ok();

        Command::new("git")
            .args(["branch", "-D", &temp_branch])
            .current_dir(parent_worktree_path)
            .status()
            .ok();
    }

    Ok(CompleteResult {
        commit_hash: final_hash,
        files_changed,
        insertions,
        deletions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_to_dirname() {
        assert_eq!(branch_to_dirname("feat/auth"), "feat-auth");
        assert_eq!(branch_to_dirname("fix/bug-123"), "fix-bug-123");
        assert_eq!(branch_to_dirname("main"), "main");
    }
}
