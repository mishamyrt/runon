#!/bin/bash
set -euo pipefail

case "$(uname -s)/$(uname -m)" in
    Darwin/arm64) arch=arm64 ;;
    Darwin/x86_64) arch=x86_64 ;;
    *) echo 'RunOn requires macOS 15 or newer on Apple Silicon or Intel.' >&2; exit 1 ;;
esac
os_version=$(sw_vers -productVersion)
if (( ${os_version%%.*} < 15 )); then
    echo 'RunOn requires macOS 15 or newer.' >&2
    exit 1
fi

repo='https://github.com/mishamyrt/runon'
if [[ $# == 0 ]]; then
    url="$repo/releases/latest/download"
elif [[ $# == 1 && $1 =~ ^v[0-9]+\.[0-9]+\.[0-9]+([.-][a-zA-Z0-9.-]+)?$ ]]; then
    url="$repo/releases/download/$1"
else
    echo "Usage: $0 [vVERSION]" >&2
    exit 1
fi

install_tmp=$(mktemp -d)
install_dir="$HOME/.local/bin"
staged_binary="$install_dir/.runon-install-$$"
trap 'rm -rf "$install_tmp"; rm -f "$staged_binary"' EXIT

archive="runon-macos-$arch.tar.gz"
curl --fail --silent --show-error --location "$url/$archive" -o "$install_tmp/$archive"
curl --fail --silent --show-error --location "$url/SHA256SUMS" -o "$install_tmp/SHA256SUMS"
(
    cd "$install_tmp"
    awk -v archive="$archive" '$2 == archive { print }' SHA256SUMS > SHA256SUMS.selected
    shasum -a 256 -c SHA256SUMS.selected
    tar -xzf "$archive"
)
mkdir -p "$install_dir"
install -m 755 "$install_tmp/runon" "$staged_binary"
mv -f "$staged_binary" "$install_dir/runon"
printf 'Installed RunOn %s to %s\n' "$("$install_dir/runon" --version)" "$install_dir/runon"
printf 'Create ~/.config/runon/config.kdl, then run: %s start\n' "$install_dir/runon"
printf 'For an existing service, apply this update with: %s restart\n' "$install_dir/runon"
