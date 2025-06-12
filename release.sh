#!/bin/bash

set -e  # Exit on any error

echo
echo "========================================"
echo "         Mathypad Release Script"
echo "========================================"
echo

# Step 1: Generate changelog preview
echo "[1/9] Generating changelog preview..."
if ! git cliff --bump > temp_changelog.md; then
    echo "ERROR: Failed to generate changelog preview"
    rm -f temp_changelog.md
    exit 1
fi

# Extract the latest version from the changelog
NEW_VERSION=$(grep "^## \[" temp_changelog.md | head -1 | sed 's/^## \[\(.*\)\].*/\1/')

if [ -z "$NEW_VERSION" ]; then
    echo "ERROR: Could not extract version from changelog"
    rm -f temp_changelog.md
    exit 1
fi

echo
echo "Found new version: $NEW_VERSION"
echo

# Step 2: Show changes for the new version
echo "[2/9] Changes in version $NEW_VERSION:"
echo "========================================"

# Extract content between the first ## and second ## (or end of file)
awk '
/^## \[/ { 
    if (found) exit
    found = 1
    next
}
found && /^## / { exit }
found { print }
' temp_changelog.md

echo "========================================"
echo

# Step 3: Ask for confirmation
read -p "Do you want to proceed with release $NEW_VERSION? (y/N): " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "Release cancelled."
    rm -f temp_changelog.md
    exit 0
fi

echo
echo "Proceeding with release $NEW_VERSION..."
echo

# Step 4: Write the changelog file
echo "[3/9] Writing changelog file..."
if ! git cliff --bump -o CHANGELOG.md; then
    echo "ERROR: Failed to write changelog file"
    rm -f temp_changelog.md
    exit 1
fi

# Step 5: Update Cargo.toml version
echo "[4/9] Updating Cargo.toml version to $NEW_VERSION..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi

# Step 6: Run cargo check to update lock file
echo "[5/9] Running cargo check to update lock file..."
if ! cargo check; then
    echo "ERROR: cargo check failed"
    rm -f temp_changelog.md
    exit 1
fi

# Step 7: Git commit
echo "[6/9] Creating git commit..."
git add CHANGELOG.md Cargo.toml Cargo.lock
if ! git commit -m "no ai: v$NEW_VERSION"; then
    echo "ERROR: Git commit failed"
    rm -f temp_changelog.md
    exit 1
fi

# Step 8: Create git tag with changelog content
echo "[7/9] Creating git tag v$NEW_VERSION..."

# Extract just the changes for this version (without the version header)
awk '
/^## \[/ { 
    if (found) exit
    found = 1
    next
}
found && /^## / { exit }
found && NF > 0 { print }
' temp_changelog.md > tag_message.tmp

if ! git tag -a "v$NEW_VERSION" -F tag_message.tmp; then
    echo "ERROR: Git tag creation failed"
    rm -f temp_changelog.md tag_message.tmp
    exit 1
fi

# Step 9: Push to remote with tags
echo "[8/9] Pushing to remote repository..."
if ! git push origin main --tags; then
    echo "ERROR: Git push failed"
    rm -f temp_changelog.md tag_message.tmp
    exit 1
fi

# Step 9.5: Clean up temporary files before cargo publish
echo "Cleaning up temporary files..."
rm -f temp_changelog.md tag_message.tmp

# Step 10: Publish to crates.io
echo "[9/9] Publishing to crates.io..."
if ! cargo publish; then
    echo "ERROR: cargo publish failed"
    echo "The commit and tag have been pushed, but crates.io publish failed."
    echo "You may need to run 'cargo publish' manually."
    exit 1
fi

echo
echo "========================================"
echo "    Release v$NEW_VERSION completed successfully!"
echo "========================================"
echo
echo "✅ Changelog updated"
echo "✅ Version bumped in Cargo.toml"
echo "✅ Git commit created"
echo "✅ Git tag created"
echo "✅ Pushed to remote repository"
echo "✅ Published to crates.io"
echo
echo "Your release is now live!"
echo "  📦 crates.io: https://crates.io/crates/mathypad"
echo "  🏷️ GitHub: https://github.com/pato/mathypad/releases/tag/v$NEW_VERSION"
echo