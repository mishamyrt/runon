#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
case "$(uname -s)/$(uname -m)" in
    Darwin/arm64) arch=arm64 ;;
    Darwin/x86_64) arch=x86_64 ;;
    *) echo 'Build on an Apple Silicon or Intel Mac.' >&2; exit 1 ;;
esac
archive="runon-macos-$arch.tar.gz"
cargo build --release --locked -p runon
version=$(target/release/runon --version)
mkdir -p dist
package_tmp=$(mktemp -d)
trap 'rm -rf "$package_tmp"' EXIT
cp target/release/runon "$package_tmp/runon"
cp LICENSE README.md "$package_tmp/"
cp -R docs "$package_tmp/docs"
mkdir "$package_tmp/examples"
cp examples/*.kdl "$package_tmp/examples/"
COPYFILE_DISABLE=1 tar -czf "dist/$archive" -C "$package_tmp" runon LICENSE README.md examples docs
(
    cd dist
    shasum -a 256 "$archive" > SHA256SUMS
    shasum -a 256 -c SHA256SUMS
)
bash build/generate-release-notes.sh "v$version" > dist/notes.md
bytes=$(stat -f '%z' target/release/runon)
if (( bytes > 5 * 1024 * 1024 )); then
    echo "Binary exceeds 5 MiB budget: $bytes bytes" >&2
    exit 1
fi
printf 'Packaged RunOn %s (%s bytes)\n' "$version" "$bytes"
