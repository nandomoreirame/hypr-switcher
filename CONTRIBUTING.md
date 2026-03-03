# Contributing to hypr-switcher

Thank you for your interest in contributing! This guide will help you get started.

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:

```bash
git clone https://github.com/<your-username>/hypr-switcher.git
cd hypr-switcher
```

3. **Install dependencies** (Arch Linux example):

```bash
sudo pacman -S wayland wayland-protocols libxkbcommon
```

4. **Build** the project:

```bash
cargo build
```

5. **Run the tests** to make sure everything works:

```bash
cargo test
```

## Making Changes

1. **Create a branch** from `develop`:

```bash
git checkout develop
git checkout -b feature/your-feature-name
```

2. **Write tests first.** This project follows TDD (Test-Driven Development). Add or update tests before writing implementation code.

3. **Implement your changes.** Keep them focused and minimal.

4. **Run the full test suite** to make sure nothing is broken:

```bash
cargo test
```

5. **Build in release mode** to catch any optimized-build issues:

```bash
cargo build --release
```

## Commit Guidelines

This project uses [Conventional Commits](https://www.conventionalcommits.org/) with emoji prefixes:

| Type | Emoji | Example |
|------|-------|---------|
| feat | ✨ | `✨ feat(ui): add grid layout` |
| fix | 🐛 | `🐛 fix(ipc): handle socket timeout` |
| docs | 📝 | `📝 docs(readme): update installation guide` |
| test | ✅ | `✅ test(app): add navigation tests` |
| refactor | ♻️ | `♻️ refactor(icons): simplify fallback chain` |
| perf | ⚡ | `⚡ perf(ipc): reduce socket read latency` |
| chore | 🔧 | `🔧 chore: update dependencies` |

Rules:
- Write commit messages in **English**, imperative mood ("add", not "added")
- Keep the first line under **72 characters**
- One logical change per commit

## Submitting a Pull Request

1. **Push** your branch to your fork:

```bash
git push origin feature/your-feature-name
```

2. **Open a Pull Request** against the `develop` branch (not `main`)

3. In the PR description, include:
   - What the change does and why
   - How to test it
   - Screenshots if it changes the UI

4. Wait for review. Address any feedback with new commits (don't force-push).

## Project Structure

```
src/
├── main.rs              # Entry point, PID management, IPC listener
├── app.rs               # Application state, update loop, keyboard handling
├── hyprland/
│   ├── ipc.rs           # Hyprland Unix socket IPC
│   └── types.rs         # Data types (HyprClient, WindowEntry)
├── icons/
│   └── resolver.rs      # XDG icon resolution with theme support
└── ui/
    ├── style.rs          # Design tokens (colors, dimensions)
    └── window_list.rs    # Window card grid rendering
```

## Testing

Run all tests:

```bash
cargo test
```

Run tests for a specific module:

```bash
cargo test app::
cargo test hyprland::
cargo test icons::
cargo test ui::
```

Run with debug logging:

```bash
RUST_LOG=debug cargo run
```

## Code of Conduct

Be respectful and constructive. We are all here to build something useful together.
