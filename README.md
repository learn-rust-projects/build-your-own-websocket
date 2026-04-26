# build-your-own-websocket

> This project is inspired by <https://github.com/tyr-rust-bootcamp/template>

## Environment Setup

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Recommended VSCode Extensions

- **Dependi**: Rust package management extension
- **CodeLLDB**: Rust debugger
- **Even Better TOML**: TOML file support
- **Better Comments**: Enhanced comment display
- **Error Lens**: Improved error highlighting
- **GitLens**: Git enhancement
- **indent-rainbow**: Indentation visualization
- **Prettier - Code formatter**: Code formatting
- **REST client**: REST API debugging
- **rust-analyzer**: Rust language support
- **Test Explorer UI**: Rust test overview
- **TODO Highlight**: TODO highlighting
- **Todo Tree**: TODO list management
- **vscode-icons**: Icon optimization
- **YAML**: YAML file support
- **Code Case Converter**: Code case conversion
- **markdownlint**: Markdown syntax checking

### Install pre-commit

pre-commit is a code quality tool that runs checks before committing code.

```bash
pipx install pre-commit
```

After installation, run `pre-commit install` to set up git hooks.

### Install Cargo deny

Cargo deny is a Cargo plugin for dependency security auditing.

```bash
cargo install --locked cargo-deny
```

### Install typos

typos is a spelling checker for code and documentation.

```bash
cargo install typos-cli
```

### Install git cliff

git cliff is a tool for generating changelogs from git history.

```bash
cargo install git-cliff
```

### Install cargo nextest

cargo nextest is an enhanced test runner for Rust projects.

```bash
cargo install cargo-nextest --locked
```

### Install cargo generate

cargo generate is a project template generator that can create new projects from existing GitHub repositories.

```bash
cargo install cargo-generate
```

## Usage

- After installing cargo generate, create a new project using this template:

```bash
cargo generate --git https://github.com/learn-rust-projects/template
```

- In the generated project, run `pre-commit install` to set up git hooks.
- Modify the `name` field in `Cargo.toml` to your project name and update the license if needed.
- Update the content in `README.md` with your project information.
- Update the repository URL in `cliff.toml` to match your project's repository.
- Make your first commit:

```bash
  git add .
  git commit -a
  # Wait for pre-commit hooks to complete
  git push
```

- generate the changelog:

```bash
  git-cliff -o CHANGELOG.md
```

- tag the release:

```bash
  git tag -a v0.1.0 -m "chore: Release wx-uploader version 0.5.1"
  git push --tags
```

## License

This project is licensed under the terms of the MIT license.
