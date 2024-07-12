#!/bin/sh
# Generate info file from template
# Usage: ./generate-info.sh <build-info-template> <version>
# Supported template variables:
# - $APP_VERSION - app version
# - $APP_BUILD_COMMIT - current commit hash
# - $APP_BUILD_DATE - current build date

set -e

if [ -z "$1" ] || [ -z "$2" ]; then
	echo "Usage: $0 <build-info-template> <version>"
	exit 1
fi

TEMPLATE_PATH="$1"
VERSION="$2"

APP_VERSION="$VERSION" \
APP_BUILD_COMMIT="$(git rev-parse --short HEAD)" \
APP_BUILD_DATE="$(date -u +"%Y-%m-%d")" \
	envsubst "\
		\$APP_VERSION \
		\$APP_BUILD_COMMIT \
		\$APP_BUILD_DATE" \
		< "$TEMPLATE_PATH"
