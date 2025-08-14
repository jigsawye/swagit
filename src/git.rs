use std::process::Command;

#[derive(Debug)]
pub struct BranchInfo {
  pub name: String,
  pub commit_id: String,
  pub worktree_path: Option<String>,
}

#[derive(Debug)]
pub enum BranchStatus {
  Updated(String),    // branch updated
  Merged(String),     // branch merged can be deleted
  RemoteGone(String), // remote branch deleted
  Diverged(String),   // local has unpushed commits
  UpToDate,           // branch is already up to date
  LocalOnly(String),  // local branch never pushed
  Modified(String),   // has uncommitted changes
}

pub struct GitManager;

impl GitManager {
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    // check if in git repository
    match Self.command("rev-parse", &["--git-dir"]) {
      Ok(_) => Ok(Self),
      Err(_) => Err("Not a git repository".into()),
    }
  }

  pub fn checkout_branch(&self, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    self.command("checkout", &[branch_name])?;
    Ok(())
  }

  pub fn delete_branches(&self, branch_names: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec!["-D"];
    args.extend(branch_names.iter().map(|s| s.as_str()));
    self.command("branch", &args)?;
    Ok(())
  }

  pub fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
    Ok(
      self
        .command("branch", &["--show-current"])?
        .trim()
        .to_string(),
    )
  }

  pub fn get_local_branches(&self) -> Result<Vec<BranchInfo>, Box<dyn std::error::Error>> {
    let current = self.get_current_branch()?;
    let output = self.command(
      "for-each-ref",
      &[
        "--format=%(refname:short) %(objectname:short)",
        "refs/heads/",
      ],
    )?;

    let worktrees = self.get_worktrees().unwrap_or_default();

    let branches = output
      .lines()
      .filter_map(|line| {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 && parts[0] != current {
          let worktree_path = worktrees.get(parts[0]).cloned();
          Some(BranchInfo {
            name: parts[0].to_string(),
            commit_id: parts[1].to_string(),
            worktree_path,
          })
        } else {
          None
        }
      })
      .collect();

    Ok(branches)
  }

  pub fn sync_branches(&self) -> Result<Vec<BranchStatus>, Box<dyn std::error::Error>> {
    // Check working directory status
    let status = self.command("status", &["--porcelain"])?;
    if !status.is_empty() {
      return Ok(vec![BranchStatus::Modified(self.get_current_branch()?)]);
    }

    // Check remote
    let remote_exists = !self.command("remote", &[])?.trim().is_empty();
    if !remote_exists {
      return Err("No remote repository configured".into());
    }

    // Update remote info
    self.command("remote", &["update", "--prune"])?;

    let mut statuses = Vec::new();
    let current = self.get_current_branch()?;
    let default_branch = self.get_default_branch()?;

    // Get all local branches
    let branches = self.get_local_branches()?;

    // Check merged branches first
    let merged_branches: Vec<String> = self
      .command("branch", &["--merged", &default_branch])?
      .lines()
      .filter_map(|line| {
        let branch = line.trim();
        if !branch.is_empty()
          && branch != &default_branch
          && branch != "master"
          && branch != "main"
          && !branch.starts_with('*')
        {
          Some(branch.to_string())
        } else {
          None
        }
      })
      .collect();

    // Process merged branches
    for branch in &merged_branches {
      self.delete_branches(&[branch.clone()])?;
      statuses.push(BranchStatus::Merged(branch.clone()));
    }

    // Check current branch status
    if !merged_branches.contains(&current) {
      statuses.push(self.check_branch_status(&current)?);
    }

    // Process other branches
    for branch in branches {
      if merged_branches.contains(&branch.name) || branch.name == current {
        continue;
      }

      // Check branch status
      statuses.push(self.check_branch_status(&branch.name)?);
    }

    Ok(statuses)
  }

  fn command(&self, cmd: &str, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").arg(cmd).args(args).output()?;

    if output.status.success() {
      Ok(String::from_utf8(output.stdout)?)
    } else {
      let error = String::from_utf8_lossy(&output.stderr);
      Err(error.into())
    }
  }

  fn check_branch_status(&self, branch: &str) -> Result<BranchStatus, Box<dyn std::error::Error>> {
    // Check if there is an upstream branch
    let has_upstream = self
      .command(
        "rev-parse",
        &["--verify", &format!("refs/remotes/origin/{}", branch)],
      )
      .is_ok();

    if !has_upstream {
      return Ok(BranchStatus::LocalOnly(branch.to_string()));
    }

    let output = self.command(
      "rev-list",
      &[
        "--left-right",
        "--count",
        &format!("{}...origin/{}", branch, branch),
      ],
    )?;

    let counts: Vec<&str> = output.trim().split_whitespace().collect();
    match counts.as_slice() {
      [left, right] => {
        let left: usize = left.parse().unwrap_or(0);
        let right: usize = right.parse().unwrap_or(0);

        match (left, right) {
          (0, 0) => Ok(BranchStatus::UpToDate),
          (_, 0) => Ok(BranchStatus::Diverged(branch.to_string())),
          (0, _) => {
            if self.check_remote_branch(branch)? {
              let local_sha = self.command("rev-parse", &[branch])?.trim().to_string();
              let remote_sha = self
                .command("rev-parse", &[&format!("origin/{}", branch)])?
                .trim()
                .to_string();

              if local_sha != remote_sha {
                self.update_branch_ref(branch, &format!("refs/remotes/origin/{}", branch))?;
                Ok(BranchStatus::Updated(branch.to_string()))
              } else {
                Ok(BranchStatus::UpToDate)
              }
            } else {
              Ok(BranchStatus::RemoteGone(branch.to_string()))
            }
          }
          _ => Ok(BranchStatus::Diverged(branch.to_string())),
        }
      }
      _ => Ok(BranchStatus::RemoteGone(branch.to_string())),
    }
  }

  fn check_remote_branch(&self, branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(
      self
        .command("rev-parse", &[&format!("refs/remotes/origin/{}", branch)])
        .is_ok(),
    )
  }

  fn get_default_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
    // Try to get from remote first
    match self.command("rev-parse", &["--abbrev-ref", "origin/HEAD"]) {
      Ok(result) if !result.trim().is_empty() => Ok(result.trim().replace("origin/", "")),
      _ => {
        // If no remote, try local common branch names
        for branch in ["main", "master"] {
          if let Ok(_) = self.command("rev-parse", &[&format!("refs/heads/{}", branch)]) {
            return Ok(branch.to_string());
          }
        }
        Err("Could not determine default branch".into())
      }
    }
  }

  fn update_branch_ref(
    &self,
    branch: &str,
    target: &str,
  ) -> Result<(), Box<dyn std::error::Error>> {
    // Use update-ref to update branch reference
    self.command("update-ref", &[&format!("refs/heads/{}", branch), target])?;
    Ok(())
  }

  pub fn get_worktrees(&self) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error>> {
    let output = match self.command("worktree", &["list", "--porcelain"]) {
      Ok(output) => output,
      Err(_) => return Ok(std::collections::HashMap::new()), // No worktrees or git version doesn't support it
    };

    let mut worktrees = std::collections::HashMap::new();
    let mut current_worktree = String::new();
    let mut current_branch = String::new();

    for line in output.lines() {
      if line.starts_with("worktree ") {
        current_worktree = line.strip_prefix("worktree ").unwrap_or("").to_string();
      } else if line.starts_with("branch ") {
        let branch_ref = line.strip_prefix("branch ").unwrap_or("");
        if let Some(branch_name) = branch_ref.strip_prefix("refs/heads/") {
          current_branch = branch_name.to_string();
        }
      }

      if !current_worktree.is_empty() && !current_branch.is_empty() {
        worktrees.insert(current_branch.clone(), current_worktree.clone());
        current_branch.clear();
      }
    }

    Ok(worktrees)
  }
}
