#!/bin/bash

if [ -z "$1" ]; then
  echo "Usage: $0 <release_version>"
  exit 1
fi

release_version=$1
changelog_file="CHANGELOG.md"

if [ ! -f "$changelog_file" ]; then
  echo "CHANGELOG.md not found!"
  exit 1
fi

sed -n "/## \[$release_version\]/,/^## \[/p" \
	"$changelog_file" | sed '$d' | sed '$d' | tail -n+2
