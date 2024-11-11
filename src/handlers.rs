use crate::git::{BranchStatus, GitManager};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, FuzzySelect};
use std::process;

pub fn handle_checkout_command(git: &GitManager) -> Result<(), Box<dyn std::error::Error>> {
  let branches = git.get_local_branches()?;

  if branches.is_empty() {
    eprintln!("{}", "Error: no other branches in the repository".red());
    process::exit(1);
  }

  let branch_names: Vec<String> = branches
    .iter()
    .map(|b| format!("{} [{}]", b.name, b.commit_id))
    .collect();

  if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
    let selection = match FuzzySelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Select the branch to switch to")
      .items(&branch_names)
      .default(0)
      .interact_opt()? {
        Some(selections) => selections,
        None => return Ok(()),
      };

    let branch_name = &branches[selection].name;
    git.checkout_branch(branch_name)?;
    println!("{}", format!("Switched to branch {}", branch_name).green());
  } else {
    let branch_name = &branches[0].name;
    git.checkout_branch(branch_name)?;
    println!("{}", format!("Switched to branch {}", branch_name).green());
  }

  Ok(())
}

pub fn handle_delete_command(git: &GitManager) -> Result<(), Box<dyn std::error::Error>> {
  let branches = git.get_local_branches()?;

  if branches.is_empty() {
    eprintln!("{}", "Error: no other branches in the repository".red());
    process::exit(1);
  }

  let branch_names: Vec<String> = branches
    .iter()
    .map(|b| format!("{} [{}]", b.name, b.commit_id))
    .collect();

  let selections = match MultiSelect::with_theme(&ColorfulTheme::default())
    .with_prompt("Select the branches to delete")
    .items(&branch_names)
    .interact_opt()? {
      Some(selections) => selections,
      None => return Ok(()),
    };

  if selections.is_empty() {
    println!("No branches selected, exiting.");
    return Ok(());
  }

  let selected_branches: Vec<String> = selections
    .iter()
    .map(|&i| branches[i].name.clone())
    .collect();

  let message = if selected_branches.len() == 1 {
    format!(
      "Are you sure you want to delete this branch?\n  {}",
      selected_branches[0]
    )
  } else {
    format!(
      "Are you sure you want to delete {} branches?\n  {}",
      selected_branches.len().to_string().yellow().bold(),
      selected_branches.join(", ")
    )
  };

  if Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(message)
    .interact()?
  {
    git.delete_branches(&selected_branches)?;
    println!(
      "{}",
      format!("Deleted {} branches", selected_branches.len()).green()
    );
  }

  Ok(())
}

pub fn handle_sync_command(git: &GitManager) -> Result<(), Box<dyn std::error::Error>> {
  println!("{}", "Syncing with remote...".blue());

  let branch_statuses = git.sync_branches()?;
  let mut has_updates = false;

  for status in branch_statuses {
    match status {
      BranchStatus::Updated(branch) => {
        has_updates = true;
        println!(
          "{} Updated branch {} (fast-forward)",
          "✓".green(),
          branch.green()
        );
      }
      BranchStatus::Merged(branch) => {
        has_updates = true;
        println!(
          "{} Deleted branch {} (was merged)",
          "✓".green(),
          branch
        );
      }
      BranchStatus::RemoteGone(branch) => {
        has_updates = true;
        println!(
          "{} Branch {} was deleted on remote but not merged",
          "!".red(),
          branch
        );
      }
      BranchStatus::Diverged(branch) => {
        has_updates = true;
        println!("{} Branch {} has unpushed commits", "!".yellow(), branch);
      }
      BranchStatus::LocalOnly(branch) => {
        println!("{} Branch {} is local only", "i".blue(), branch);
      }
      BranchStatus::Modified(branch) => {
        has_updates = true;
        println!(
          "{} Branch {} has uncommitted changes",
          "!".yellow(),
          branch
        );
      }
      BranchStatus::UpToDate => ()
    }
  }

  if !has_updates {
    println!("{}", "Everything is up to date".green());
  }

  Ok(())
}
