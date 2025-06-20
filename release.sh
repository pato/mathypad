#!/bin/bash

set -e  # Exit on any error

# Show usage if help requested
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Mathypad Release Script"
    echo
    echo "Usage:"
    echo "  ./release.sh           Run the full release process"
    echo "  ./release.sh --dry-run Preview what the release would do without making changes"
    echo "  ./release.sh --help    Show this help message"
    echo
    echo "The script will:"
    echo "  1. Check for uncommitted changes (cargo publish will fail if any exist)"
    echo "  2. Generate a changelog preview and show upcoming changes"
    echo "  3. Ask for confirmation before proceeding"
    echo "  4. Update CHANGELOG.md and Cargo.toml version"
    echo "  5. Build the project with embedded changelog"
    echo "  6. Create a git commit and tag"
    echo "  7. Push to remote repository"
    echo "  8. Publish to crates.io"
    echo
    exit 0
fi

# Check for dry run mode
DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo
    echo "========================================"
    echo "    Mathypad Release Script (DRY RUN)"
    echo "========================================"
    echo "🔍 This is a dry run - no changes will be made"
    echo
else
    echo
    echo "========================================"
    echo "         Mathypad Release Script"
    echo "========================================"
    echo
fi

# Step 0: Check for uncommitted changes
echo "[0/10] Checking for uncommitted changes..."
if [ "$DRY_RUN" = true ]; then
    if ! git diff-index --quiet HEAD --; then
        echo "🔍 DRY RUN: Found uncommitted changes that would prevent release:"
        git status --porcelain
        echo "🔍 DRY RUN: In a real release, this would cause the script to exit"
        echo "🔍 DRY RUN: Please commit these changes before running the actual release"
    else
        echo "✅ Working directory is clean"
    fi
else
    if ! git diff-index --quiet HEAD --; then
        echo "ERROR: You have uncommitted changes in your working directory."
        echo
        echo "The following files have uncommitted changes:"
        git status --porcelain
        echo
        echo "Please commit or stash these changes before running the release script."
        echo "cargo publish will fail if there are any uncommitted changes."
        exit 1
    fi
    echo "✅ Working directory is clean"
fi

# Step 1: Generate changelog preview
echo "[1/10] Generating changelog preview..."
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
echo "[2/10] Changes in version $NEW_VERSION:"
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
echo "[3/10] Writing changelog file..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would write changelog to CHANGELOG.md using: git cliff --bump -o CHANGELOG.md"
else
    if ! git cliff --bump -o CHANGELOG.md; then
        echo "ERROR: Failed to write changelog file"
        rm -f temp_changelog.md
        exit 1
    fi
fi

# Step 5: Update Cargo.toml version
echo "[4/10] Updating Cargo.toml version to $NEW_VERSION..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would update Cargo.toml version to $NEW_VERSION"
    echo "🔍 DRY RUN: Current version line: $(grep '^version = ' Cargo.toml)"
    echo "🔍 DRY RUN: Would become: version = \"$NEW_VERSION\""
else
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    fi
fi

# Step 6: Build project to ensure changelog is embedded and update lock file
echo "[5/10] Building project with updated changelog..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would run: cargo build"
    echo "🔍 DRY RUN: This would embed the updated changelog in the binary"
else
    if ! cargo build; then
        echo "ERROR: cargo build failed"
        rm -f temp_changelog.md
        exit 1
    fi
fi

# Verify the changelog is embedded correctly
echo "Verifying changelog is embedded in binary..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would verify that ./target/debug/mathypad --changelog contains version $NEW_VERSION"
    echo "🔍 DRY RUN: Current embedded changelog shows:"
    if [ -f "./target/debug/mathypad" ]; then
        ./target/debug/mathypad --changelog | head -10
    else
        echo "🔍 DRY RUN: No existing binary found at ./target/debug/mathypad"
    fi
else
    if ! ./target/debug/mathypad --changelog | grep -q "## \[$NEW_VERSION\]"; then
        echo
        echo "⚠️  WARNING: Changelog verification failed!"
        echo "The binary doesn't seem to contain version $NEW_VERSION in its embedded changelog."
        echo
        echo "This could mean:"
        echo "  - The changelog wasn't properly embedded during build"
        echo "  - There's a caching issue with the build"
        echo "  - The include_str! macro isn't working as expected"
        echo
        echo "Current embedded changelog shows:"
        ./target/debug/mathypad --changelog | head -10
        echo
        read -p "Do you want to continue with the release anyway? (y/N): " confirm_changelog
        if [[ ! "$confirm_changelog" =~ ^[Yy]$ ]]; then
            echo "Release cancelled due to changelog verification failure."
            rm -f temp_changelog.md
            exit 1
        fi
        echo "Continuing with release despite changelog verification failure..."
    else
        echo "✅ Changelog verification passed - version $NEW_VERSION found in binary"
    fi
fi

# Step 7: Git commit
echo "[6/10] Creating git commit..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would run: git add CHANGELOG.md Cargo.toml Cargo.lock"
    echo "🔍 DRY RUN: Would commit with message: 'no ai: v$NEW_VERSION'"
    echo "🔍 DRY RUN: Files that would be staged:"
    echo "  - CHANGELOG.md"
    echo "  - Cargo.toml" 
    echo "  - Cargo.lock"
else
    git add CHANGELOG.md Cargo.toml Cargo.lock
    if ! git commit -m "no ai: v$NEW_VERSION"; then
        echo "ERROR: Git commit failed"
        rm -f temp_changelog.md
        exit 1
    fi
fi

# Step 8: Create git tag with changelog content
echo "[7/10] Creating git tag v$NEW_VERSION..."

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

if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would create tag v$NEW_VERSION with the following message:"
    echo "---"
    cat tag_message.tmp
    echo "---"
    echo "🔍 DRY RUN: Command would be: git tag -a \"v$NEW_VERSION\" -F tag_message.tmp"
else
    if ! git tag -a "v$NEW_VERSION" -F tag_message.tmp; then
        echo "ERROR: Git tag creation failed"
        rm -f temp_changelog.md tag_message.tmp
        exit 1
    fi
fi

# Step 9: Push to remote with tags
echo "[8/10] Pushing to remote repository..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would run: git push origin main --tags"
    echo "🔍 DRY RUN: This would push the commit and tag v$NEW_VERSION to origin/main"
else
    if ! git push origin main --tags; then
        echo "ERROR: Git push failed"
        rm -f temp_changelog.md tag_message.tmp
        exit 1
    fi
fi

# Step 9.5: Clean up temporary files before cargo publish
echo "Cleaning up temporary files..."
rm -f temp_changelog.md tag_message.tmp

# Step 10: Publish to crates.io
echo "[10/10] Publishing to crates.io..."
if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN: Would run: cargo publish"
    echo "🔍 DRY RUN: This would publish version $NEW_VERSION to crates.io"
else
    if ! cargo publish; then
        echo "ERROR: cargo publish failed"
        echo "The commit and tag have been pushed, but crates.io publish failed."
        echo "You may need to run 'cargo publish' manually."
        exit 1
    fi
fi

echo
if [ "$DRY_RUN" = true ]; then
    echo "========================================"
    echo "  Dry Run Complete - v$NEW_VERSION"
    echo "========================================"
    echo
    echo "🔍 The following actions would be performed:"
    echo "✅ Check for uncommitted changes (must be clean)"
    echo "✅ Update CHANGELOG.md with new changes"
    echo "✅ Bump version to $NEW_VERSION in Cargo.toml"
    echo "✅ Build project with embedded changelog"
    echo "✅ Create git commit: 'no ai: v$NEW_VERSION'"
    echo "✅ Create git tag: v$NEW_VERSION"
    echo "✅ Push commit and tag to origin/main"
    echo "✅ Publish v$NEW_VERSION to crates.io"
    echo
    echo "To perform the actual release, run:"
    echo "  ./release.sh"
else
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
fi
echo