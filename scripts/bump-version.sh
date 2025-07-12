#!/bin/bash
# Helper script to bump versions in the workspace

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_usage() {
    echo "Usage: $0 [patch|minor|major] [--core-only|--main-only]"
    echo ""
    echo "Arguments:"
    echo "  patch      Bump patch version (0.1.0 -> 0.1.1)"
    echo "  minor      Bump minor version (0.1.0 -> 0.2.0)"
    echo "  major      Bump major version (0.1.0 -> 1.0.0)"
    echo ""
    echo "Options:"
    echo "  --core-only    Only bump mathypad-core version"
    echo "  --main-only    Only bump mathypad version"
    echo ""
    echo "Examples:"
    echo "  $0 patch           # Bump patch version for both crates"
    echo "  $0 minor --core-only  # Bump minor version for mathypad-core only"
}

if [ $# -eq 0 ] || [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    print_usage
    exit 0
fi

BUMP_TYPE=$1
SCOPE=${2:-"all"}

# Validate bump type
if [[ ! "$BUMP_TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo -e "${RED}Error: Invalid bump type '$BUMP_TYPE'${NC}"
    print_usage
    exit 1
fi

# Function to bump version
bump_version() {
    local current=$1
    local type=$2
    
    IFS='.' read -r major minor patch <<< "$current"
    
    case $type in
        patch)
            patch=$((patch + 1))
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Function to update version in Cargo.toml
update_cargo_version() {
    local file=$1
    local new_version=$2
    local crate_name=$3
    
    # Update the version field
    sed -i.bak -E "/^\[package\]/,/^\[/ s/version = \"[^\"]+\"/version = \"$new_version\"/" "$file"
    rm "${file}.bak"
    
    echo -e "${GREEN}✓${NC} Updated $crate_name to version $new_version"
}

# Get current versions
CORE_VERSION=$(grep '^version' mathypad-core/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
MAIN_VERSION=$(grep '^version' Cargo.toml | grep -A1 '\[package\]' | grep version | sed 's/.*"\(.*\)".*/\1/')

echo "Current versions:"
echo "  mathypad-core: $CORE_VERSION"
echo "  mathypad: $MAIN_VERSION"
echo ""

# Calculate new versions
NEW_CORE_VERSION=$(bump_version "$CORE_VERSION" "$BUMP_TYPE")
NEW_MAIN_VERSION=$(bump_version "$MAIN_VERSION" "$BUMP_TYPE")

# Update versions based on scope
if [ "$SCOPE" != "--main-only" ]; then
    echo "Bumping mathypad-core: $CORE_VERSION -> $NEW_CORE_VERSION"
    update_cargo_version "mathypad-core/Cargo.toml" "$NEW_CORE_VERSION" "mathypad-core"
    
    # Also update the dependency version in the main crate
    sed -i.bak -E "s/mathypad-core = \{ version = \"[^\"]+\"/mathypad-core = { version = \"$NEW_CORE_VERSION\"/" Cargo.toml
    rm Cargo.toml.bak
    
    # Update web-poc dependency
    if [ -f "web-poc/Cargo.toml" ]; then
        sed -i.bak -E "s/mathypad-core = \{ version = \"[^\"]+\"/mathypad-core = { version = \"$NEW_CORE_VERSION\"/" web-poc/Cargo.toml
        rm web-poc/Cargo.toml.bak
    fi
fi

if [ "$SCOPE" != "--core-only" ]; then
    echo "Bumping mathypad: $MAIN_VERSION -> $NEW_MAIN_VERSION"
    update_cargo_version "Cargo.toml" "$NEW_MAIN_VERSION" "mathypad"
fi

echo ""
echo -e "${GREEN}Version bump complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Review the changes: git diff"
echo "2. Run tests: cargo test --all"
echo "3. Run the release script: ./scripts/release.sh"