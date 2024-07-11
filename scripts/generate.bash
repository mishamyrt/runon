#!/bin/bash

TPL_FILE="Sources/BuildInfo.swift.dist"
OUT_FILE="Sources/BuildInfo.swift"

if [ ! -f "$TPL_FILE" ]; then
	echo "File not found: $TPL_FILE"
	exit 1
elif [ -z "$1" ]; then
	echo "Version not specified"
	exit 1
fi

set -e

APP_VERSION=${1} \
BUILD_COMMIT="$(git rev-parse --short HEAD)" \
BUILD_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
	envsubst "\$APP_VERSION \$BUILD_COMMIT \$BUILD_DATE" < "$TPL_FILE" > "$OUT_FILE"

