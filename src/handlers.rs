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
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
      .with_prompt("Select the branch to switch to")
      .items(&branch_names)
      .default(0)
      .interact()?;

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

  let selections = MultiSelect::with_theme(&ColorfulTheme::default())
    .with_prompt("Select the branches to delete")
    .items(&branch_names)
    .interact()?;

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

  // sync all branches
  let branch_statuses = git.sync_all_branches()?;
  for status in branch_statuses {
    match status {
      BranchStatus::Updated(branch) => {
        println!(
          "{} Updated branch {} (fast-forward)",
          "✓".green(),
          branch.green()
        );
      }
      BranchStatus::Merged(branch) => {
        println!(
          "{} Branch {} was merged to default branch",
          "!".yellow(),
          branch
        );
      }
      BranchStatus::RemoteGone(branch) => {
        println!(
          "{} Branch {} was deleted on remote but not merged",
          "!".red(),
          branch
        );
      }
      BranchStatus::Diverged(branch) => {
        println!("{} Branch {} has unpushed commits", "!".yellow(), branch);
      }
    }
  }

  // get and process merged branches
  let merged_branches = git.get_merged_branches()?;

  if !merged_branches.is_empty() {
    println!(
      "\nFound {} merged branches that can be deleted:",
      merged_branches.len()
    );
    for branch in &merged_branches {
      println!("  {} [{}]", branch.name, branch.commit_id);
    }

    if Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt("Do you want to delete these merged branches?")
      .interact()?
    {
      let branch_names: Vec<String> = merged_branches.into_iter().map(|b| b.name).collect();

      git.delete_branches(&branch_names)?;
      println!(
        "{}",
        format!("Deleted {} merged branches", branch_names.len()).green()
      );
    }
  } else {
    println!("\n{}", "No merged branches to clean up".blue());
  }

  Ok(())
}
