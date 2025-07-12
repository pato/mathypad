#!/bin/bash
# Robust release script for mathypad workspace
# Can resume from specific steps if failures occur

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}[STEP $1]${NC} $2"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

# Function to check if a tag exists
tag_exists() {
    git tag | grep -q "^$1$"
}

# Function to check if a crate version is already published
crate_published() {
    local crate=$1
    local version=$2
    cargo search "$crate" --limit 1 | grep -q "$crate = \"$version\""
}

# Get current versions
MATHYPAD_CORE_VERSION=$(grep '^version' mathypad-core/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
MATHYPAD_VERSION=$(grep '^version' Cargo.toml | grep -A1 '\[package\]' | grep version | sed 's/.*"\(.*\)".*/\1/')

# Define release steps
STEPS=(
    "check_git_status"
    "run_tests"
    "check_versions"
    "commit_version_bumps"
    "create_tags"
    "publish_mathypad_core"
    "publish_mathypad"
    "push_to_git"
)

# Parse command line arguments
START_STEP=1
if [ $# -gt 0 ]; then
    if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
        echo "Usage: $0 [--from-step N] [--list-steps]"
        echo ""
        echo "Options:"
        echo "  --from-step N    Resume from step N (1-${#STEPS[@]})"
        echo "  --list-steps     List all available steps"
        echo ""
        echo "Examples:"
        echo "  $0                    # Run all steps"
        echo "  $0 --from-step 5      # Resume from step 5 (create_tags)"
        echo "  $0 --list-steps       # Show all steps"
        exit 0
    elif [ "$1" == "--list-steps" ]; then
        echo "Available steps:"
        for i in "${!STEPS[@]}"; do
            echo "  $((i+1)). ${STEPS[$i]}"
        done
        exit 0
    elif [ "$1" == "--from-step" ]; then
        if [ -z "$2" ]; then
            print_error "Please specify a step number"
            exit 1
        fi
        START_STEP=$2
        if [ $START_STEP -lt 1 ] || [ $START_STEP -gt ${#STEPS[@]} ]; then
            print_error "Invalid step number. Must be between 1 and ${#STEPS[@]}"
            exit 1
        fi
        print_warning "Resuming from step $START_STEP: ${STEPS[$((START_STEP-1))]}"
    fi
fi

# Step implementations

check_git_status() {
    print_step 1 "Checking git status"
    
    if ! git diff --quiet || ! git diff --cached --quiet; then
        print_error "You have uncommitted changes. Please commit or stash them first."
        exit 1
    fi
    
    print_success "Git working directory is clean"
}

run_tests() {
    print_step 2 "Running tests"
    
    cargo test --all
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt --all -- --check
    
    print_success "All tests passed"
}

check_versions() {
    print_step 3 "Checking versions"
    
    echo "Current versions:"
    echo "  mathypad-core: $MATHYPAD_CORE_VERSION"
    echo "  mathypad: $MATHYPAD_VERSION"
    
    # Check if versions are already published
    if crate_published "mathypad-core" "$MATHYPAD_CORE_VERSION"; then
        print_warning "mathypad-core $MATHYPAD_CORE_VERSION is already published on crates.io"
        echo "You may need to bump the version in mathypad-core/Cargo.toml"
    fi
    
    if crate_published "mathypad" "$MATHYPAD_VERSION"; then
        print_warning "mathypad $MATHYPAD_VERSION is already published on crates.io"
        echo "You may need to bump the version in Cargo.toml"
    fi
    
    read -p "Continue with these versions? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Release cancelled"
        exit 1
    fi
}

commit_version_bumps() {
    print_step 4 "Committing version bumps"
    
    # Check if there are any changes to commit
    if git diff --quiet; then
        print_warning "No version changes to commit, skipping"
    else
        git add -A
        git commit -m "chore: release mathypad-core $MATHYPAD_CORE_VERSION, mathypad $MATHYPAD_VERSION"
        print_success "Version bumps committed"
    fi
}

create_tags() {
    print_step 5 "Creating git tags"
    
    # Create tags for each crate
    CORE_TAG="mathypad-core-v$MATHYPAD_CORE_VERSION"
    MAIN_TAG="v$MATHYPAD_VERSION"
    
    # Check and create mathypad-core tag
    if tag_exists "$CORE_TAG"; then
        print_warning "Tag $CORE_TAG already exists, skipping"
    else
        git tag -a "$CORE_TAG" -m "Release mathypad-core $MATHYPAD_CORE_VERSION"
        print_success "Created tag $CORE_TAG"
    fi
    
    # Check and create mathypad tag
    if tag_exists "$MAIN_TAG"; then
        print_warning "Tag $MAIN_TAG already exists, skipping"
    else
        git tag -a "$MAIN_TAG" -m "Release mathypad $MATHYPAD_VERSION"
        print_success "Created tag $MAIN_TAG"
    fi
}

publish_mathypad_core() {
    print_step 6 "Publishing mathypad-core to crates.io"
    
    if crate_published "mathypad-core" "$MATHYPAD_CORE_VERSION"; then
        print_warning "mathypad-core $MATHYPAD_CORE_VERSION already published, skipping"
    else
        cd mathypad-core
        cargo publish
        cd ..
        print_success "Published mathypad-core $MATHYPAD_CORE_VERSION"
        
        # Wait a bit for crates.io to process
        print_warning "Waiting 30 seconds for crates.io to process..."
        sleep 30
    fi
}

publish_mathypad() {
    print_step 7 "Publishing mathypad to crates.io"
    
    if crate_published "mathypad" "$MATHYPAD_VERSION"; then
        print_warning "mathypad $MATHYPAD_VERSION already published, skipping"
    else
        cargo publish
        print_success "Published mathypad $MATHYPAD_VERSION"
    fi
}

push_to_git() {
    print_step 8 "Pushing to git remote"
    
    echo "Ready to push the following:"
    echo "  - Current branch to origin"
    echo "  - Tags: $CORE_TAG, $MAIN_TAG"
    
    read -p "Push to remote? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git push
        git push --tags
        print_success "Pushed to remote"
    else
        print_warning "Skipped pushing to remote"
        echo "You can manually push later with:"
        echo "  git push"
        echo "  git push --tags"
    fi
}

# Main execution
echo "🚀 Mathypad Release Script"
echo "========================="
echo ""

# Execute steps starting from START_STEP
for i in $(seq $((START_STEP-1)) $((${#STEPS[@]}-1))); do
    ${STEPS[$i]}
    echo ""
done

echo "✨ Release process completed!"
echo ""
echo "If you need to re-run any steps, use:"
echo "  $0 --from-step N"
echo ""
echo "Next steps:"
echo "1. Check that packages are visible on crates.io:"
echo "   https://crates.io/crates/mathypad-core"
echo "   https://crates.io/crates/mathypad"
echo "2. Create a GitHub release if desired"