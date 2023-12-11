#!/usr/bin/env bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Find git-aware template files (ignores things like node_modules, etc.)
# and copy them to dist/templates
git ls-files ../templates/ | rsync --files-from=- ../ dist

# Replace all Rooch package links with mustache placeholder used by create-create-app
# that will be replaced with the latest Rooch version number when the template is used
find ./dist/templates/* -name "package.json" -type f | while read -r file; do
  echo "Before replacement in $file:"
  cat "$file"
  perl -pi -e 's|"(?=@roochnetwork)([^"]+)":\s*"link:[^"]+"|"\1": "{{rooch-version}}"|g' "$file"
  echo "After replacement in $file:"
  cat "$file"
  echo
done

# Check if any files still have "link:" dependencies
if grep -r -E 'link:' ./dist/templates; then
  echo "Linked dependencies found in dist/templates"
  exit 1
fi

# Since npm-packlist does not include ".gitignore" files in the packaging process,
# these files are renamed to "gitignore".
# create-create-app automatically renames them back to ".gitignore" upon execution.
find ./dist/templates/* -name ".gitignore" -type f | while read -r file; do
  newfile="$(dirname "$file")/gitignore"
  echo "Renaming $file to $newfile"
  mv "$file" "$newfile"
done
