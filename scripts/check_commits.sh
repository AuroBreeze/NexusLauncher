#!/bin/bash

# scripts/check_commits.sh
# Check if commit messages follow the Conventional Commits specification

# Base branch to compare against (default: origin/main)
BASE_BRANCH=${1:-origin/main}

# Conventional Commits Regex
REGEXP="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?: .+$"

echo "Checking commit messages between $BASE_BRANCH and HEAD..."

# Get all commit messages in the current branch that are not in the base branch
# We use a process substitution to read each line safely
mapfile -t commits < <(git log "$BASE_BRANCH..HEAD" --no-merges --format=%s)

if [ ${#commits[@]} -eq 0 ]; then
    echo "No new commits found to check."
    exit 0
fi

FAILED_COUNT=0
TOTAL_COUNT=${#commits[@]}

echo "Found $TOTAL_COUNT commit(s) to validate."
echo "--------------------------------------"

for msg in "${commits[@]}"; do
    if [[ ! $msg =~ $REGEXP ]]; then
        echo -e "\033[0;31m[FAIL] \"$msg\"\033[0m"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    else
        echo -e "\033[0;32m[PASS] \"$msg\"\033[0m"
    fi
done

echo "--------------------------------------"
if [ $FAILED_COUNT -gt 0 ]; then
    echo -e "\033[0;31mError: $FAILED_COUNT of $TOTAL_COUNT commit message(s) failed validation.\033[0m"
    echo "Expected format: <type>(optional scope): <description>"
    echo "Example types: feat, fix, docs, refactor, chore, etc."
    exit 1
else
    echo -e "\033[0;32mAll $TOTAL_COUNT commit message(s) are valid!\033[0m"
    exit 0
fi
