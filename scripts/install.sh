#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
prefix="${PREFIX:-$HOME/.local}"
data="${XDG_DATA_HOME:-$HOME/.local/share}"

say() { printf '\033[1;34m::\033[0m %s\n' "$*"; }

say "building OmaNod (release)"
cargo build --release --locked --manifest-path "$root/Cargo.toml" -p omanod

say "installing the binary into $prefix/bin"
install -Dm755 "$root/target/release/omanod" "$prefix/bin/omanod"

say "installing icons into $data/icons/hicolor"
install -Dm644 "$root/packaging/icons/omanod.svg" "$data/icons/hicolor/scalable/apps/omanod.svg"
for size in 16 24 32 48 64 128 256 512; do
  install -Dm644 "$root/packaging/icons/hicolor/${size}x${size}/apps/omanod.png" \
    "$data/icons/hicolor/${size}x${size}/apps/omanod.png"
done

say "installing the desktop entry"
install -Dm644 "$root/packaging/omanod.desktop" "$data/applications/omanod.desktop"

command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$data/icons/hicolor" || true
command -v update-desktop-database >/dev/null && update-desktop-database -q "$data/applications" || true

say "done — press SUPER + MENU and search for OmaNod"
case ":$PATH:" in
  *":$prefix/bin:"*) ;;
  *) say "note: add $prefix/bin to PATH to run it from a terminal" ;;
esac
