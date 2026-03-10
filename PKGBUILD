# Maintainer: Haydon Ryan <haydon.ryan@gmail.com>

pkgname=ebook2audiobook
pkgver=0.2.1
pkgrel=1
pkgdesc='Convert EPUB books into text files for audiobooks'
arch=('x86_64')
url='https://github.com/haydonryan/epub2audiobook'
license=('MIT')
makedepends=('cargo')
optdepends=('ffmpeg: for WAV to MP3 encoding with bundled script')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/haydonryan/epub2audiobook/archive/v${pkgver}.tar.gz")
sha256sums=('eb3ad5a1e84d8d47e6603966a046cc791491239f15ab7b539c4b9969ee0cbb61')

build() {
  cd "${srcdir}/epub2audiobook-${pkgver}"
  cargo build --release --locked
}

package() {
  cd "${srcdir}/epub2audiobook-${pkgver}"
  install -Dm755 target/release/ebook2audiobook "${pkgdir}/usr/bin/ebook2audiobook"
  install -Dm644 LICENSE.txt "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE.txt"
}
