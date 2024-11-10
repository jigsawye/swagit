use std::process::Command;

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub commit_id: String,
}

#[derive(Debug)]
pub enum BranchStatus {
    Updated(String),    // 分支已更新
    Merged(String),     // 分支已合併可刪除
    RemoteGone(String), // 遠端分支已刪除
    Diverged(String),   // 本地有未推送的提交
}

pub struct GitManager;

impl GitManager {
    pub fn new() -> Result<Self, String> {
        // 檢查是否在 git 倉庫中
        let status = Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .output()
            .map_err(|e| e.to_string())?;

        if status.status.success() {
            Ok(Self)
        } else {
            Err("Not a git repository".to_string())
        }
    }

    pub fn get_current_branch(&self) -> Result<String, String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8(output.stdout)
                .map(|s| s.trim().to_string())
                .map_err(|e| e.to_string())
        } else {
            Err("Failed to get current branch".to_string())
        }
    }

    pub fn get_local_branches(&self) -> Result<Vec<BranchInfo>, String> {
        let output = Command::new("git")
            .args(["for-each-ref", "--format=%(refname:short) %(objectname:short)", "refs/heads/"])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("Failed to get branches".to_string());
        }

        let current = self.get_current_branch()?;
        let branches = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 && parts[0] != current {
                    Some(BranchInfo {
                        name: parts[0].to_string(),
                        commit_id: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(branches)
    }

    pub fn checkout_branch(&self, branch_name: &str) -> Result<(), String> {
        let status = Command::new("git")
            .args(["checkout", branch_name])
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to checkout branch {}", branch_name))
        }
    }

    pub fn delete_branches(&self, branch_names: &[String]) -> Result<(), String> {
        let mut args = vec!["branch", "-D"];
        args.extend(branch_names.iter().map(|s| s.as_str()));

        let status = Command::new("git")
            .args(&args)
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to delete branches".to_string())
        }
    }

    pub fn sync_all_branches(&self) -> Result<Vec<BranchStatus>, String> {
        // Fetch all branches and prune deleted remote branches
        let status = Command::new("git")
            .args(["fetch", "--prune", "origin"])
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to fetch from remote".to_string());
        }

        let mut statuses = Vec::new();
        let branches = self.get_local_branches()?;
        let current = self.get_current_branch()?;

        for branch in branches {
            // Check if the remote branch exists
            let remote_exists = Command::new("git")
                .args(["rev-parse", "--verify", &format!("origin/{}", branch.name)])
                .status()
                .map_err(|e| e.to_string())?
                .success();

            if !remote_exists {
                if self.is_branch_merged(&branch.name)? {
                    statuses.push(BranchStatus::Merged(branch.name));
                } else {
                    statuses.push(BranchStatus::RemoteGone(branch.name));
                }
                continue;
            }

            // Check branch status
            let output = Command::new("git")
                .args(["rev-list", "--left-right", "--count", &format!("{}...origin/{}", branch.name, branch.name)])
                .output()
                .map_err(|e| e.to_string())?;

            if output.status.success() {
              let output_str = String::from_utf8(output.stdout)
                    .map_err(|e| e.to_string())?;

                let counts = output_str
                    .trim()
                    .split_whitespace()
                    .collect::<Vec<_>>();

                if counts.len() == 2 {
                    let ahead: usize = counts[0].parse().unwrap_or(0);
                    let behind: usize = counts[1].parse().unwrap_or(0);

                    match (ahead, behind) {
                        (0, 0) => continue, // Branch is synced
                        (0, _) => {
                            // Local is behind, can fast-forward
                            if branch.name == current {
                                Command::new("git")
                                    .args(["merge", "--ff-only", &format!("origin/{}", branch.name)])
                                    .status()
                                    .map_err(|e| e.to_string())?;
                            }
                            statuses.push(BranchStatus::Updated(branch.name));
                        }
                        (_, _) => statuses.push(BranchStatus::Diverged(branch.name)),
                    }
                }
            }
        }

        Ok(statuses)
    }

    fn get_default_branch(&self) -> Result<String, String> {
        // Try to get the default branch from origin/HEAD
        let output = Command::new("git")
            .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let branch = String::from_utf8(output.stdout)
                .map_err(|e| e.to_string())?
                .trim()
                .replace("refs/remotes/origin/", "");
            return Ok(branch);
        }

        // If the above fails, try to check main or master
        for branch in ["main", "master"] {
            let exists = Command::new("git")
                .args(["rev-parse", "--verify", branch])
                .status()
                .map_err(|e| e.to_string())?
                .success();

            if exists {
                return Ok(branch.to_string());
            }
        }

        // If nothing is found, return an error
        Err("Could not determine default branch".to_string())
    }

    fn is_branch_merged(&self, branch: &str) -> Result<bool, String> {
        let default_branch = self.get_default_branch()?;
        let output = Command::new("git")
            .args(["branch", "--merged", &default_branch])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let merged = String::from_utf8(output.stdout)
                .map_err(|e| e.to_string())?
                .lines()
                .map(|line| line.trim().trim_start_matches('*').trim())
                .any(|line| line == branch);
            Ok(merged)
        } else {
            Err("Failed to check merged branches".to_string())
        }
    }

    pub fn get_merged_branches(&self) -> Result<Vec<BranchInfo>, String> {
        let default_branch = self.get_default_branch()?;
        let output = Command::new("git")
            .args(["branch", "--merged", &default_branch, "--format=%(refname:short) %(objectname:short)"])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("Failed to get merged branches".to_string());
        }

        let current = self.get_current_branch()?;
        let branches = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 && parts[0] != default_branch && parts[0] != current {
                    Some(BranchInfo {
                        name: parts[0].to_string(),
                        commit_id: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(branches)
    }
}
