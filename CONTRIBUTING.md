# Contributing to Nexus Launcher

Thank you for your interest in improving Nexus Launcher! To keep the project maintainable and high-quality, please follow these guidelines.

## Where to Start

- **Check the Task List**: We maintain an automatically generated [todo_list.md](todo_list.md) which aggregates all `TODO`, `FIXME`, and `PERF` comments from the codebase.
- **Explore the Code**: Searching for `TODO` in your editor is a fantastic way to find pending tasks, optimizations, and known minor issues.

## Development Workflow

We use several tools to ensure code quality and streamline the contribution process.

### 1. Pre-commit Hooks
We use the `pre-commit` framework to run checks automatically before every commit.

**Prerequisites:**
- **Python 3** and **pip** are required to run the framework.
- **Installation**:
    - **Linux**: You can usually install it via your package manager:
      ```bash
      sudo apt install pre-commit  # Debian/Ubuntu
      sudo pacman -S pre-commit    # Arch Linux
      sudo dnf install pre-commit   # Fedora
      ```
    - **Any OS (via pip)**:
      ```bash
      pip install pre-commit
      ```

**Setup**:
Once installed, run `./scripts/setup_hooks.sh` to activate the hooks in this repository.

**What it does**:
- It automatically checks formatting (`cargo fmt`) and runs lints (`cargo clippy`).
- It updates the [todo_list.md](todo_list.md) automatically and stages it.
- It enforces [Conventional Commits](https://www.conventionalcommits.org/) during the `commit-msg` stage.

### 2. Pre-push Checks
Before pushing your changes, we recommend running our comprehensive check script:
```bash
./scripts/check.sh
```
This runs formatting checks, clippy, all tests, builds the project, and validates your commit history.

### 3. Pull Requests
- **Reviewers**: When you open a PR, our automation will automatically CC previous contributors of the modified files to request a review.
- **CI**: Every PR triggers a CI pipeline that runs the same checks as `check.sh`. All checks must pass before merging.

## How to Contribute

1.  **Report Bugs**: Open an issue describing the bug, your environment, and steps to reproduce it.
2.  **Suggest Features**: Open an issue to discuss new ideas before implementation.
3.  **Submit Pull Requests**:
    * Fork the repository and create your branch from `main`.
    * Ensure your code follows the existing style.
    * Use **Conventional Commit** messages.
    * Open a PR with a clear description of your changes.

## Development Standards

* **Rust Version**: Use the latest stable Rust.
* **Testing**: Add tests for new features and ensure all existing tests pass with `cargo test`.
* **Security**: Use `cargo audit` to check for vulnerable dependencies.

## Getting Help

If you have questions, feel free to open an issue for discussion or reach out in our community [Discord](https://discord.gg/gM85PKSYEe).
