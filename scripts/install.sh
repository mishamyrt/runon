#!/bin/bash
set -euo pipefail

if [[ $(uname -s) != Darwin || $(uname -m) != arm64 ]]; then
    echo 'RunOn requires Apple Silicon and macOS 26 or newer.' >&2
    exit 1
fi
os_version=$(sw_vers -productVersion)
if (( ${os_version%%.*} < 26 )); then
    echo 'RunOn requires macOS 26 or newer.' >&2
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

curl --fail --silent --show-error --location "$url/runon-macos-arm64.tar.gz" -o "$install_tmp/runon-macos-arm64.tar.gz"
curl --fail --silent --show-error --location "$url/SHA256SUMS" -o "$install_tmp/SHA256SUMS"
(
    cd "$install_tmp"
    shasum -a 256 -c SHA256SUMS
    tar -xzf runon-macos-arm64.tar.gz
)
mkdir -p "$install_dir"
install -m 755 "$install_tmp/runon" "$staged_binary"
mv -f "$staged_binary" "$install_dir/runon"
printf 'Installed RunOn %s to %s\n' "$("$install_dir/runon" --version)" "$install_dir/runon"
printf 'Create ~/.config/runon/config.kdl, then run: %s start\n' "$install_dir/runon"
printf 'For an existing service, apply this update with: %s restart\n' "$install_dir/runon"
