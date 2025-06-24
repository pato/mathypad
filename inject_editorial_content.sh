#!/bin/bash

# Script to inject editorial content into changelog while preserving existing editorial sections
# Usage: ./inject_editorial_content.sh VERSION "EDITORIAL_CONTENT" INPUT_CHANGELOG OUTPUT_CHANGELOG

set -e

if [ $# -ne 4 ]; then
    echo "Usage: $0 VERSION 'EDITORIAL_CONTENT' INPUT_CHANGELOG OUTPUT_CHANGELOG"
    exit 1
fi

VERSION="$1"
EDITORIAL_CONTENT="$2"
INPUT_CHANGELOG="$3"
OUTPUT_CHANGELOG="$4"

# Create a temporary file for processing
TEMP_OUTPUT=$(mktemp)

# Flag to track processing state
IN_TARGET_VERSION=false
FOUND_TARGET_VERSION=false
ADDED_EDITORIAL=false

# Read the input changelog line by line
while IFS= read -r line; do
    # Check if this is a version header
    if [[ "$line" =~ ^##\ \[([0-9]+\.[0-9]+\.[0-9]+)\] ]]; then
        # Extract version from the line
        LINE_VERSION="${BASH_REMATCH[1]}"
        
        if [ "$LINE_VERSION" = "$VERSION" ]; then
            # This is our target version
            IN_TARGET_VERSION=true
            FOUND_TARGET_VERSION=true
            echo "$line" >> "$TEMP_OUTPUT"
        else
            # This is a different version
            # If we were in target version and haven't added editorial yet, add it now
            if [ "$IN_TARGET_VERSION" = true ] && [ "$ADDED_EDITORIAL" = false ] && [ -n "$EDITORIAL_CONTENT" ]; then
                echo "" >> "$TEMP_OUTPUT"
                echo "### 🌟 User Notes" >> "$TEMP_OUTPUT"
                echo "" >> "$TEMP_OUTPUT"
                # Add each line of editorial content
                echo "$EDITORIAL_CONTENT" | while IFS= read -r editorial_line; do
                    if [ -n "$editorial_line" ]; then
                        echo "$editorial_line" >> "$TEMP_OUTPUT"
                    fi
                done
                echo "" >> "$TEMP_OUTPUT"
                ADDED_EDITORIAL=true
            fi
            IN_TARGET_VERSION=false
            echo "$line" >> "$TEMP_OUTPUT"
        fi
    elif [ "$IN_TARGET_VERSION" = true ] && [[ "$line" =~ ^###\ 🌟\ User\ Notes ]]; then
        # Skip existing User Notes section header - we'll replace it
        IN_USER_NOTES=true
        continue
    elif [ "$IN_TARGET_VERSION" = true ] && [[ "$line" =~ ^### ]] && [ "${IN_USER_NOTES:-false}" = true ]; then
        # We've hit a different section after User Notes, so add our editorial content now
        if [ "$ADDED_EDITORIAL" = false ] && [ -n "$EDITORIAL_CONTENT" ]; then
            echo "### 🌟 User Notes" >> "$TEMP_OUTPUT"
            echo "" >> "$TEMP_OUTPUT"
            # Add each line of editorial content
            echo "$EDITORIAL_CONTENT" | while IFS= read -r editorial_line; do
                if [ -n "$editorial_line" ]; then
                    echo "$editorial_line" >> "$TEMP_OUTPUT"
                fi
            done
            echo "" >> "$TEMP_OUTPUT"
            ADDED_EDITORIAL=true
        fi
        IN_USER_NOTES=false
        echo "$line" >> "$TEMP_OUTPUT"
    elif [ "$IN_TARGET_VERSION" = true ] && [ "${IN_USER_NOTES:-false}" = true ]; then
        # Skip existing user notes content - we'll replace it
        continue
    elif [ "$IN_TARGET_VERSION" = true ] && [[ "$line" =~ ^### ]] && [ "$ADDED_EDITORIAL" = false ] && [ -n "$EDITORIAL_CONTENT" ]; then
        # This is the first section after the version header, add editorial content before it
        echo "" >> "$TEMP_OUTPUT"
        echo "### 🌟 User Notes" >> "$TEMP_OUTPUT"
        echo "" >> "$TEMP_OUTPUT"
        # Add each line of editorial content
        echo "$EDITORIAL_CONTENT" | while IFS= read -r editorial_line; do
            if [ -n "$editorial_line" ]; then
                echo "$editorial_line" >> "$TEMP_OUTPUT"
            fi
        done
        echo "" >> "$TEMP_OUTPUT"
        ADDED_EDITORIAL=true
        echo "$line" >> "$TEMP_OUTPUT"
    else
        # Regular line - just copy it
        echo "$line" >> "$TEMP_OUTPUT"
    fi
done < "$INPUT_CHANGELOG"

# If we reached the end and were still in target version, add editorial content
if [ "$IN_TARGET_VERSION" = true ] && [ "$ADDED_EDITORIAL" = false ] && [ -n "$EDITORIAL_CONTENT" ]; then
    echo "" >> "$TEMP_OUTPUT"
    echo "### 🌟 User Notes" >> "$TEMP_OUTPUT"
    echo "" >> "$TEMP_OUTPUT"
    # Add each line of editorial content
    echo "$EDITORIAL_CONTENT" | while IFS= read -r editorial_line; do
        if [ -n "$editorial_line" ]; then
            echo "$editorial_line" >> "$TEMP_OUTPUT"
        fi
    done
    echo "" >> "$TEMP_OUTPUT"
fi

# If we didn't find the target version, something went wrong
if [ "$FOUND_TARGET_VERSION" = false ]; then
    echo "ERROR: Could not find version $VERSION in changelog"
    rm -f "$TEMP_OUTPUT"
    exit 1
fi

# Move the temporary file to the output location
mv "$TEMP_OUTPUT" "$OUTPUT_CHANGELOG"

echo "✅ Editorial content injected successfully for version $VERSION"