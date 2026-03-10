# AUR Maintainer Guide (ebook2audiobook)

This repo includes a ready-to-publish AUR package definition under `aur/ebook2audiobook/`.

## Prerequisites
- An AUR account and SSH key configured.
- `git` and `base-devel` installed.

## Where the AUR files live
- `aur/ebook2audiobook/PKGBUILD`
- `aur/ebook2audiobook/.SRCINFO`

## Publish / Update Flow

1. Ensure `pkgver` and `sha256sums` are correct.
   - `pkgver` should match the tag (example `0.2.1` for tag `v0.2.1`).
   - Update the `sha256sums` from the GitHub release tarball.

2. Regenerate `.SRCINFO` if you edit `PKGBUILD`.
   From `aur/ebook2audiobook/`:
   ``` bash
   makepkg --printsrcinfo > .SRCINFO
   ```

3. Push to the AUR git repo.
   ``` bash
   git clone ssh://aur@aur.archlinux.org/ebook2audiobook.git
   cd ebook2audiobook
   cp /home/h/workspace/epub2audiobook/aur/ebook2audiobook/PKGBUILD .
   cp /home/h/workspace/epub2audiobook/aur/ebook2audiobook/.SRCINFO .
   git add PKGBUILD .SRCINFO
   git commit -m "Update to v<version>"
   git push
   ```

## Local Build (no AUR publish required)
From this repo:
``` bash
cd /home/h/workspace/epub2audiobook/aur/ebook2audiobook
makepkg -si
```

## Version Bump Checklist
- Tag the release in git (e.g., `v0.2.2`).
- Update `pkgver` in `PKGBUILD`.
- Update the `source` URL if needed (usually just the version).
- Update `sha256sums`.
- Regenerate `.SRCINFO`.
- Commit and push to AUR.
