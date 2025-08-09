# SQL CLI Release Guide

## Building Release Binaries

### Local Release Build
```bash
# Build optimized release binary
cargo build --release

# The binary will be at: target/release/sql-cli
```

### Cross-Platform Builds

To build for multiple platforms, install `cross`:
```bash
cargo install cross
```

Then build for different targets:
```bash
# Linux x86_64 (most common)
cross build --release --target x86_64-unknown-linux-gnu

# Linux ARM64 (for ARM servers/Raspberry Pi)
cross build --release --target aarch64-unknown-linux-gnu

# macOS x86_64
cross build --release --target x86_64-apple-darwin

# macOS ARM64 (M1/M2)
cross build --release --target aarch64-apple-darwin

# Windows
cross build --release --target x86_64-pc-windows-gnu
```

## Reducing Binary Size

To create smaller binaries:

1. Add to `Cargo.toml`:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Reduce parallel codegen for better optimization
strip = true        # Strip symbols
```

2. Use UPX for additional compression (optional):
```bash
# Install UPX
sudo apt-get install upx  # On Ubuntu/Debian

# Compress the binary
upx --best --lzma target/release/sql-cli
```

## Creating a Release

1. **Version Bump**: Update version in `Cargo.toml`
```toml
[package]
name = "sql-cli"
version = "0.2.0"  # Update this
```

2. **Tag the Release**:
```bash
git tag -a v0.2.0 -m "Release v0.2.0: ORDER BY support and smart completions"
git push origin v0.2.0
```

3. **GitHub Release**:
```bash
# Using GitHub CLI
gh release create v0.2.0 \
  --title "v0.2.0: ORDER BY Support" \
  --notes "## Features
- ORDER BY execution in cache/CSV mode
- Smart tab completion for ORDER BY
- Intelligent column width auto-sizing
- Type-aware sorting for all data types

## Improvements
- Fix ORDER BY context detection
- Only suggest selected columns in ORDER BY completion
- Better performance with optimized column calculations" \
  target/release/sql-cli
```

## Distribution Options

### 1. Direct Binary Distribution
- Upload to GitHub Releases
- Users download and add to PATH

### 2. Homebrew (macOS/Linux)
Create a formula:
```ruby
class SqlCli < Formula
  desc "Enhanced SQL CLI with intelligent completions"
  homepage "https://github.com/yourusername/sql-cli"
  url "https://github.com/yourusername/sql-cli/releases/download/v0.2.0/sql-cli-darwin-x64.tar.gz"
  sha256 "YOUR_SHA256_HERE"
  version "0.2.0"

  def install
    bin.install "sql-cli"
  end
end
```

### 3. Cargo Install
Publish to crates.io:
```bash
cargo publish
```

Users can then install with:
```bash
cargo install sql-cli
```

### 4. Linux Package Managers
- Create `.deb` for Debian/Ubuntu
- Create `.rpm` for RedHat/Fedora
- Submit to AUR for Arch Linux

## Testing Release Binary

Before distributing:
```bash
# Test the release binary
./target/release/sql-cli --version
./target/release/sql-cli cache sample_trades.json
./target/release/sql-cli --help

# Test on different systems if possible
```

## Recommended Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run all tests: `cargo test`
- [ ] Build release binary: `cargo build --release`
- [ ] Test release binary manually
- [ ] Create git tag
- [ ] Push tag to GitHub
- [ ] Create GitHub release
- [ ] Upload binaries for multiple platforms
- [ ] Update documentation/README
- [ ] Announce release (if applicable)