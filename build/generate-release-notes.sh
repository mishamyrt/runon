#!/bin/bash
set -euo pipefail
if [[ $# != 1 ]]; then
    echo "Usage: $0 vVERSION" >&2
    exit 1
fi
awk -v version="$1" '
    /^## \[/ {
        if (found) exit
        if (index($0, "[" version "]")) { found = 1; next }
    }
    found { print }
    END { if (!found) exit 1 }
' CHANGELOG.md
