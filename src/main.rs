mod git;
mod handlers;

use clap::{Arg, Command};
use colored::*;
use dialoguer::console::Term;
use git::GitManager;
use handlers::{handle_checkout_command, handle_delete_command, handle_sync_command};
use std::process;

fn main() {
  if let Err(err) = ctrlc::set_handler(move || {
    let _ = Term::stdout().show_cursor();
    process::exit(0);
  }) {
    eprintln!("{}", format!("Error setting Ctrl-C handler: {}", err).red());
    process::exit(1);
  }

  let matches = Command::new("swagit")
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::new("delete")
        .short('d')
        .long("delete")
        .help("Select branches which you want to delete")
        .action(clap::ArgAction::SetTrue),
    )
    .arg(
      Arg::new("sync")
        .short('s')
        .long("sync")
        .help("Pull latest changes and cleanup merged branches")
        .action(clap::ArgAction::SetTrue),
    )
    .get_matches();

  let git = match GitManager::new() {
    Ok(git) => git,
    Err(_) => {
      eprintln!("{}", "Error: not a git repository".red());
      process::exit(1);
    }
  };

  match git.get_current_branch() {
    Ok(branch) => println!("{} Current branch is {}", "Info:".blue(), branch.magenta()),
    Err(_) => {
      eprintln!("{}", "Error: could not get current branch".red());
      process::exit(1);
    }
  }

  let result = match (matches.get_flag("delete"), matches.get_flag("sync")) {
    (true, _) => handle_delete_command(&git),
    (_, true) => handle_sync_command(&git),
    _ => handle_checkout_command(&git),
  };

  if let Err(err) = result {
    if !err.to_string().contains("read interrupted") {
      eprintln!("{}", format!("Error: {}", err).red());
      process::exit(1);
    }
    process::exit(0);
  }
}
