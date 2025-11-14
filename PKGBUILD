# Maintainer: Nico Valianto Kusuma <YOUR_EMAIL@gmail.com>

pkgname=declarch
pkgver=0.1.0
pkgrel=1
pkgdesc="A declarative package management CLI for Arch Linux, inspired by Nix."
arch=('x86_64')
url="https://github.com/nixval/declarch"
license=('MIT')
depends=('pacman')
optdepends=(
  'paru: AUR backend for syncing'
  'yay: alternative AUR backend'
)
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP') # Replace with real checksum using 'updpkgsums'

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
