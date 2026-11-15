#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
version=$(<../VERSION)
target=x86_64-pc-windows-gnu
mkdir -p "target/$target" ../installers
resource="$PWD/target/$target/app-icon.o"
printf '1 ICON "assets/app-icon.ico"\n' | x86_64-w64-mingw32-windres -J rc -O coff -o "$resource"
cargo rustc --locked --release --target "$target" --bin game-of-life-bevy -- -C "link-arg=$resource"
makensis -V2 "-DVERSION=$version" scripts/package-windows.nsi
