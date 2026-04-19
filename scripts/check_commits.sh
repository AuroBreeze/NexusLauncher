#!/bin/bash

# scripts/check_commits.sh
# Check if commit messages follow the Conventional Commits specification

# Base branch to compare against (default: origin/main)
BASE_BRANCH=${1:-origin/main}

# Conventional Commits Regex
# Type: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
REGEXP="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?: .+$"

echo "Checking commit messages between $BASE_BRANCH and HEAD..."

# Get all commit messages in the current branch that are not in the base branch
# --no-merges to skip merge commits
commits=$(git log $BASE_BRANCH..HEAD --no-merges --format=%s)

if [ -z "$commits" ]; then
    echo "No new commits found to check."
    exit 0
fi

FAILED=0
while IFS= read -r msg; do
    if [[ ! $msg =~ $REGEXP ]]; then
        echo -e "\033[0;31mInvalid commit message: \"$msg\"\033[0m"
        FAILED=1
    else
        echo -e "\033[0;32mValid commit message: \"$msg\"\033[0m"
    fi
done <<< "$commits"

if [ $FAILED -eq 1 ]; then
    echo -e "\n\033[0;31mError: Some commit messages do not follow the convention.\033[0m"
    echo "Expected format: <type>(optional scope): <description>"
    echo "Examples: feat: add auth, fix(ui): fix button alignment, docs: update readme"
    exit 1
fi

echo -e "\n\033[0;32mAll commit messages are valid!\033[0m"
