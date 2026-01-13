# Declarch Release Guide

This guide explains how to publish a new release of declarch.

## Prerequisites

1. Configure SSH keys for AUR:
   ```bash
   # Add your AUR SSH key
   ssh-keygen -t ed25519 -f ~/.ssh/aur
   # Add the public key to https://aur.archlinux.org/account
   ```

2. Install required tools:
   ```bash
   # Arch Linux
   sudo pacman -S git openssh
   
   # Build tools
   sudo pacman -S base-devel
   ```

## Release Process

### 1. Update Version

Update version in `Cargo.toml`:
```toml
[package]
name = "declarch"
version = "0.4.0"  # Update this
```

### 2. Build and Test

```bash
# Build release binary
cargo build --release

# Run tests
cargo test

# Test the binary
./target/release/declarch --version
./target/release/declarch sync --dry-run
```

### 3. Commit Changes

```bash
git add Cargo.toml
git commit -m "bump: version 0.4.0"
```

### 4. Create and Push Tag

```bash
# Create annotated tag
git tag -a v0.4.0 -m "Release v0.4.0"

# Push tag to trigger GitHub Actions
git push origin main
git push origin v0.4.0
```

### 5. GitHub Actions Builds

The GitHub Actions workflow will automatically:
- Build x86_64 and aarch64 binaries
- Create a GitHub release
- Upload binaries as release assets

Check the Actions tab: https://github.com/nixval/declarch/actions

### 6. Publish to AUR

Once the GitHub release is created:

```bash
# From the declarch repository
./.aur/publish.sh 0.4.0
```

This will:
- Prepare PKGBUILD files
- Generate .SRCINFO files
- Publish to `declarch` AUR package
- Publish to `declarch-bin` AUR package

### 7. Verify AUR Packages

After publishing, verify the packages:
```bash
# Test build (optional)
git clone ssh://aur@aur.archlinux.org/declarch.git
cd declarch
makepkg -si

# Or install from AUR
paru -S declarch
paru -S declarch-bin
```

## Installation Methods

After release, users can install declarch using:

### Via Install Script (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install-remote.sh | sh
```

### Via AUR (Arch Linux)
```bash
# Build from source
paru -S declarch

# Precompiled binary
paru -S declarch-bin
```

### Via Cargo
```bash
cargo install declarch
```

### From GitHub Release
```bash
# Download binary
wget https://github.com/nixval/declarch/releases/download/v0.4.0/declarch-0.4.0-x86_64-unknown-linux-gnu.tar.gz

# Extract and install
tar -xzf declarch-0.4.0-x86_64-unknown-linux-gnu.tar.gz
sudo install declarch /usr/local/bin/
```

## Troubleshooting

### AUR SSH Access Issues

If you get "Permission denied" when pushing to AUR:
```bash
# Test SSH connection
ssh -T aur@aur.archlinux.org

# Check which key is being used
ssh -vT aur@aur.archlinux.org

# Add to ~/.ssh/config:
Host aur.archlinux.org
    IdentityFile ~/.ssh/aur
    User aur
```

### GitHub Actions Failures

Check the build logs in the Actions tab. Common issues:
- Version mismatch between Cargo.toml and tag
- Cross-compilation errors for aarch64
- Missing release assets

### PKGBUILD Validation

```bash
# Validate PKGBUILD syntax
namcap PKGBUILD

# Test build
makepkg -sf
```

## Post-Release Checklist

- [ ] Version updated in Cargo.toml
- [ ] Tag pushed to GitHub
- [ ] GitHub Actions build successful
- [ ] Release created with binaries
- [ ] AUR packages published
- [ ] AUR packages build successfully
- [ ] Installation tested (all methods)
- [ ] Documentation updated

## Rollback (If Needed)

If something goes wrong:

```bash
# Delete GitHub release
gh release delete v0.4.0 --yes

# Delete tag
git tag -d v0.4.0
git push origin :refs/tags/v0.4.0

# Update AUR (with fixed version)
./.aur/publish.sh 0.4.1
```
