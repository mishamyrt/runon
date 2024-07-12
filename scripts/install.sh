#!/bin/bash

set -e

# Handle release argument
if [ -z "$1" ]; then
	RELEASE="latest"
else
	RELEASE="tag/$1"
fi

# Setup temp dir and cleanup
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

# Internal variables
PROJECT_REPO="https://github.com/mishamyrt/runon"
DIST_URL="$PROJECT_REPO/releases/$RELEASE/download/runon.zip"
DIST_FILE="$TEMP_DIR/runon.zip"
INSTALLATION_DIR="/usr/local/bin"
SCRIPT_FILE_NAME="runon"
DAEMON_FILE_NAME="runon-daemon"
SCRIPT_PATH="$INSTALLATION_DIR/$SCRIPT_FILE_NAME"
DAEMON_PATH="$INSTALLATION_DIR/$DAEMON_FILE_NAME"

colored_print() {
    echo -e "\033[${2}m${1}\033[0m"
}

grey() {
    colored_print "$1" "30"
}

green() {
    colored_print "$1" "32"
}

# Function to color text in yellow
yellow() {
    colored_print "$1" "33"
}

# Function to color text in blue
blue() {
    colored_print "$1" "34"
}

# Function to color text in magenta
magenta() {
    colored_print "$1" "35"
}

get_dist() {
	curl -sL "$DIST_URL" -o "$DIST_FILE"
	unzip "$DIST_FILE" -d "$TEMP_DIR" > /dev/null
}

install_dist() {
	sudo rm -f \
		"$SCRIPT_PATH" \
		"$DAEMON_PATH"
	sudo cp "$TEMP_DIR/$SCRIPT_FILE_NAME" "$SCRIPT_PATH"
	sudo cp "$TEMP_DIR/$DAEMON_FILE_NAME" "$DAEMON_PATH"
}

install() {
	echo "Installing runon $RELEASE"
	previous_version="$(runon-daemon --version || echo "")"
	if [ -n "$previous_version" ]; then
		grey "Overwriting previous version: $previous_version"
	fi
	grey "Downloading distribution..."
	get_dist
	grey "Installing..."
	install_dist
	green "Success"
}

install

