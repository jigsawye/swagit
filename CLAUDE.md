# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Swagit is an interactive Git CLI tool written in Rust that provides a fuzzy-searchable interface for common Git operations. The tool was rewritten from Node.js/TypeScript to Rust starting from version 1.0.0 for better performance.

## Architecture

### Core Components

- **GitManager** (`src/git.rs`): Core Git operations wrapper that handles all Git command execution and provides methods for branch management, status checking, and synchronization
- **Handlers** (`src/handlers.rs`): Command-specific implementations for the three main operations:
  - `handle_checkout_command`: Interactive branch selection and switching
  - `handle_delete_command`: Multi-select branch deletion with confirmation
  - `handle_sync_command`: Remote sync with automatic cleanup of merged branches
- **Main** (`src/main.rs`): CLI setup, argument parsing, and command routing

### Key Data Structures

- `BranchInfo`: Contains branch name and commit ID
- `BranchStatus` enum: Tracks various branch states (Updated, Merged, RemoteGone, Diverged, UpToDate, LocalOnly, Modified)

## Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run specific test file
cargo test --test git_tests
cargo test --test integration_tests

# Build release version
cargo build --release

# Format code
cargo fmt

# Run clippy linting
cargo clippy
```


## Key Implementation Details

### Branch Synchronization Logic
The sync functionality (`sync_branches()` in GitManager) performs a complex workflow:
1. Checks for uncommitted changes
2. Updates remote references with `git remote update --prune`
3. Identifies merged branches using `git branch --merged`
4. Automatically deletes merged branches
5. Checks status of remaining branches using `git rev-list --left-right --count`

### Interactive CLI Features
- Uses `dialoguer` crate for interactive prompts with fuzzy search
- Handles both interactive (TTY) and non-interactive modes
- Graceful Ctrl-C handling with proper cursor cleanup

### Distribution
The project is distributed as a Rust binary via crates.io.

## Common Patterns

- Error handling uses `Box<dyn std::error::Error>` throughout
- Git commands are executed via `std::process::Command` and wrapped in the `GitManager::command()` method
- All user-facing output uses the `colored` crate for terminal colors
- Interactive prompts check for TTY before presenting UI

## Testing Strategy

Tests are located in `tests/` directory and use:
- `assert_cmd` for CLI testing
- `tempfile` for creating temporary Git repositories
- `predicates` for output assertions
- Test setup creates full Git repositories with proper configuration