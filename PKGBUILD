# Maintainer: nixval <nicovaliantoku@gmail.com>

pkgname=declarch
pkgver=0.4.0
pkgrel=1
pkgdesc="A declarative package manager for Arch Linux, inspired by Nix workflow"
arch=('x86_64')
url="https://github.com/nixval/declarch"
license=('MIT')
depends=('pacman' 'git')
optdepends=(
  'paru: AUR backend for syncing'
  'yay: Alternative AUR helper'
  'flatpak: For managing Flatpak applications'
)
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('GENERATE_WITH_MAKEPKG')  # Run: makepkg -g

prepare() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo build --release --frozen
}

check() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

  # Install shell completions
  install -Dm644 "target/release/build/$pkgname"-*/out/"$pkgname".bash \
    "$pkgdir/usr/share/bash-completion/completions/$pkgname"
  install -Dm644 "target/release/build/$pkgname"-*/out/"$pkgname".fish \
    "$pkgdir/usr/share/fish/vendor_completions.d/$pkgname".fish
  install -Dm644 "target/release/build/$pkgname"-*/out/"$pkgname".zsh \
    "$pkgdir/usr/share/zsh/site-functions/_$pkgname"
}
