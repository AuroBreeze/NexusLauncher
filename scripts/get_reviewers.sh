#!/bin/bash

# scripts/get_reviewers.sh
# BASE_BRANCH: the branch to compare against (default origin/main)
# EXCLUDE_USER: the email or name to exclude from the list (the PR author)

BASE_BRANCH=${1:-origin/main}
EXCLUDE_USER=${2:-""}

# Get list of modified files
files=$(git diff --name-only $BASE_BRANCH...HEAD)

if [ -z "$files" ]; then
    exit 0
fi

# Collect contributors, excluding the current author
potential_reviewers=$(
    for file in $files; do
        if [ -f "$file" ]; then
            # Get top contributors' names/emails
            git shortlog -sne -- "$file" | awk '{print $NF}' | sed 's/[<>]//g'
        fi
    done | grep -v "$EXCLUDE_USER" | sort | uniq -c | sort -nr | head -n 3 | awk '{print $2}'
)

# Output reviewers separated by commas
echo "$potential_reviewers" | tr '\n' ',' | sed 's/,$//'
