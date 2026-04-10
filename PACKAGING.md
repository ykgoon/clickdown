# Packaging Guide for ClickDown

This document provides instructions for packaging ClickDown for Arch Linux and Debian-based distributions.

## Table of Contents
- [Arch Linux (AUR)](#arch-linux-aur)
- [Debian/Ubuntu (.deb)](#debianubuntu-deb)
- [Updating Versions](#updating-versions)
- [Testing Packages](#testing-packages)

---

## Arch Linux (AUR)

### Package Location
AUR package files are maintained in the `.aur/` directory of this repository.

### Files Included
- `PKGBUILD` - Build script for the AUR package
- `.SRCINFO` - Package metadata for AUR
- `clickdown.desktop` - Desktop entry file for integration

### Installing from AUR

Users can install ClickDown using AUR helpers:

```bash
# Using yay
yay -S clickdown-bin

# Using paru
paru -S clickdown-bin

# Using manual AUR build
git clone https://aur.archlinux.org/clickdown-bin.git
cd clickdown-bin
makepkg -si
```

### Submitting to AUR

To submit the package to AUR:

1. Ensure you have an AUR account and SSH keys configured
2. Clone the AUR repository:
   ```bash
   git clone ssh://aur@aur.archlinux.org/clickdown-bin.git
   ```
3. Copy files from `.aur/` directory:
   ```bash
   cp .aur/PKGBUILD clickdown-bin/
   cp .aur/.SRCINFO clickdown-bin/
   cp .aur/clickdown.desktop clickdown-bin/
   ```
4. Commit and push:
   ```bash
   cd clickdown-bin
   git add .
   git commit -m "Initial package release"
   git push
   ```

### Updating the AUR Package

When releasing a new version:

1. Update `pkgver` in `PKGBUILD`
2. Update the source URL with new version
3. Generate new checksums:
   ```bash
   updpkgsums
   ```
4. Update `.SRCINFO`:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```
5. Commit and push to AUR

---

## Debian/Ubuntu (.deb)

### Automated Builds

The `.deb` package is automatically built and attached to GitHub releases via GitHub Actions. No manual packaging required.

### Manual Build (for testing)

If you need to build a `.deb` package locally:

```bash
# Install dependencies
sudo apt-get install -y dpkg-dev debhelper

# Create package structure
VERSION=$(cargo pkgid | cut -d# -f2 | cut -d: -f2)
mkdir -p deb-pkg/DEBIAN
mkdir -p deb-pkg/usr/bin
mkdir -p deb-pkg/usr/share/doc/clickdown
mkdir -p deb-pkg/usr/share/applications

# Build binary
cargo build --release
cp target/release/clickdown deb-pkg/usr/bin/

# Create control file
cat > deb-pkg/DEBIAN/control << EOF
Package: clickdown
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Depends: libssl3
Maintainer: Y.K. Goon
Description: A fast and responsive ClickUp desktop client for the terminal
 ClickDown is a terminal-based ClickUp client built with Rust and ratatui.
EOF

# Create desktop entry
cp .aur/clickdown.desktop deb-pkg/usr/share/applications/

# Build the package
dpkg-deb --build --root-owner-group deb-pkg clickdown_${VERSION}_amd64.deb
```

### Installing the .deb Package

```bash
# Download from GitHub releases
wget https://github.com/ykgoon/clickdown/releases/download/v${VERSION}/clickdown_${VERSION}_amd64.deb

# Install
sudo dpkg -i clickdown_${VERSION}_amd64.deb

# Fix any dependency issues
sudo apt-get install -f
```

### Verifying the Package

```bash
# Check package info
dpkg -s clickdown

# List installed files
dpkg -L clickdown

# Remove package
sudo dpkg -r clickdown
```

---

## Updating Versions

### For New Releases

When releasing a new version (e.g., `v0.3.0`):

1. **Update GitHub Actions**: No changes needed - it automatically uses the tag version
2. **Update AUR Package**:
   - Change `pkgver=0.3.0` in `PKGBUILD`
   - Update source URL to new release
   - Run `updpkgsums` to update checksums
   - Run `makepkg --printsrcinfo > .SRCINFO` to update metadata
3. **Tag the release**: `git tag v0.3.0 && git push origin v0.3.0`
4. GitHub Actions will automatically build and attach packages

### Version Synchronization

Ensure version numbers match across:
- `Cargo.toml` (source of truth)
- `.aur/PKGBUILD` (`pkgver`)
- `.aur/.SRCINFO` (`pkgver`)
- GitHub release tag (format: `vX.Y.Z`)

---

## Testing Packages

### Testing Arch Linux Package

```bash
# Build locally
cd .aur
makepkg -s

# Install locally
sudo pacman -U clickdown-bin-*.pkg.tar.zst

# Test
clickdown --help

# Remove
sudo pacman -R clickdown-bin
```

### Testing Debian Package

```bash
# Build locally (see Manual Build section above)

# Install
sudo dpkg -i clickdown_*.deb

# Test
clickdown --help

# Remove
sudo dpkg -r clickdown
```

### CI/CD Testing

The GitHub Actions workflow automatically:
1. Builds the binary for multiple platforms
2. Creates the `.deb` package
3. Attaches all artifacts to the release
4. No manual intervention required

---

## Troubleshooting

### Common Issues

**AUR: Checksum mismatch**
```bash
# Update checksums
updpkgsums
```

**Debian: Missing dependencies**
```bash
# Fix dependencies
sudo apt-get install -f
```

**Debian: Package conflicts**
```bash
# Remove old version first
sudo dpkg -r clickdown
sudo dpkg -i clickdown_new.deb
```

**Desktop entry not showing**
```bash
# Update desktop database
sudo update-desktop-database
```

---

## Contributing

If you'd like to help maintain these packages:

1. Fork the repository
2. Make changes to `.aur/` files or GitHub Actions workflow
3. Submit a pull request
4. Test your changes locally before submitting

---

## Links

- [AUR Package](https://aur.archlinux.org/packages/clickdown-bin) (after submission)
- [GitHub Releases](https://github.com/ykgoon/clickdown/releases)
- [Arch Wiki - PKGBUILD](https://wiki.archlinux.org/title/PKGBUILD)
- [Debian Packaging Tutorial](https://www.debian.org/doc/manuals/maintainer-guide/)
