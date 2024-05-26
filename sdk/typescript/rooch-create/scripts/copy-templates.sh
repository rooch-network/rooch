#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Find git-aware template files (ignores things like node_modules, etc.)
# and copy them to dist/templates
git ls-files ../templates/ | rsync --files-from=- ../ dist

# Function to get the latest version from NPM
get_latest_version() {
  local package_name=$1
  npm show "$package_name" version
}

# Find all package.json files
find ./dist/templates/* -name "package.json" -type f | while read -r file; do
  echo "Before replacement in $file:"
  cat "$file"

  # Read the package.json content
  content=$(cat "$file")

  # Extract all @roochnetwork packages with link: or workspace: versions
  packages=$(echo "$content" | jq -r '
    .dependencies // {} | to_entries |
    map(select(.key | test("^@roochnetwork/")) | select(.value | test("^link:|^workspace:"))) |
    .[].key
  ')

  if [ -z "$packages" ]; then
    echo "No @roochnetwork packages to update."
    continue
  fi

  updated_content=$content

  # Iterate over each package to update its version
  for package in $packages; do
    latest_version=$(get_latest_version "$package")
    updated_content=$(echo "$updated_content" | jq --arg package "$package" --arg version "$latest_version" '
      .dependencies[$package] = $version
    ')
  done

  echo "$updated_content" > "$file"

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
