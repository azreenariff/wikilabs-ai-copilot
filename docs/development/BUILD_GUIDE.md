# Build Guide — Wiki Labs AI Copilot

## Prerequisites

### Required

- **Rust**: Stable toolchain (via `rustup`)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup default stable
  cargo --version  # >= 1.75
  ```

- **Cargo clippy** and **rustfmt** (included with rustup):
  ```bash
  rustup component add clippy rustfmt
  ```

### Optional (for full development)

- **Node.js 20+** (for Tauri frontend, if building the desktop app)
  ```bash
  # Install via nvm or your preferred package manager
  nvm install 20
  npm install -g pnpm
  ```

- **Tauri CLI** (for desktop app builds):
  ```bash
  npm install -g @tauri-apps/cli
  ```

- **Platform-specific tools**:
  - **Windows**: Visual Studio Build Tools (C++ workload)
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `build-essential`, `libssl-dev`, `libsqlite3-dev`

## Building

### Core Engine Only (Rust workspace)

```bash
# Clean build
cargo clean

# Build all modules
cargo build --all

# Release build
cargo build --release --all
```

### Desktop App (Tauri + Frontend)

```bash
# Build frontend
cd src-tauri/frontend
pnpm install
pnpm build

# Build desktop app
cd ../..
cargo tauri build
```

## Testing

```bash
# Unit tests
cargo test --all --lib

# Integration tests
cargo test --all --test '*'

# With coverage
cargollvm-cov llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html
```

## Linting

```bash
# Check formatting
cargo fmt --all --check

# Run clippy (exit on warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix what clippy can
cargo clippy --all-targets --all-features --fix --allow-dirty
```

## CI/CD

GitHub Actions runs automatically on PR and push to `main`:

- `.github/workflows/pr.yml` — PR checks (format, clippy, build, test)
- `.github/workflows/main.yml` — Main branch CI (build, test, coverage)
- `.github/workflows/release.yml` — Release workflow (tag-based build + GitHub release)

## Platform-Specific Notes

### Windows

- Requires Visual Studio Build Tools (C++ workload).
- SQLite bundled via `rusqlite` feature `bundled` (links to SQLite3).

### macOS

- Requires Xcode Command Line Tools.
- Code signing: `codesign --sign <signing-identity> target/release/wikilabs`

### Linux

- Install dependencies:
  ```bash
  # Ubuntu/Debian
  sudo apt install build-essential libssl-dev libsqlite3-dev

  # RHEL/Fedora
  sudo dnf install gcc openssl-devel sqlite-devel
  ```
- AppImage requires `linuxdeploy` and `linuxdeploy-plugin-appimage`:
  ```bash
  wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
  chmod +x linuxdeploy-x86_64.AppImage
  ./linuxdeploy-x86_64.AppImage --appimage-extract
  ```