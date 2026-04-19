#!/bin/bash

# Get involved contributors for modified files
# Similar to Linux kernel's get_maintainer.pl approach

# Color definitions
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get list of modified, staged, or untracked files
files=$(git status --porcelain | awk '{print $2}' | sort -u)

# If arguments are provided, use them instead
if [ $# -gt 0 ]; then
  files="$@"
fi

if [ -z "$files" ]; then
  echo -e "${YELLOW}No modified files detected.${NC}"
  echo "Usage: $0 [file1 file2 ...]"
  exit 0
fi

echo -e "${CYAN}Analyzing top contributors for the following files:${NC}\n"

for file in $files; do
  # Check if the file exists (it might have been deleted)
  if [ ! -f "$file" ]; then
    continue
  fi

  echo -e "${YELLOW}File: $file${NC}"

  # Analyze git history for this specific file
  # -s: summary (suppress commit description)
  # -n: sort by number of commits
  # -e: show email address
  # head -n 5: show top 5 contributors
  git shortlog -sne -- "$file" | head -n 5 | sed 's/^/  /'

  # Optional: Show when the file was last modified
  last_mod=$(git log -1 --format="%ai (%ar)" -- "$file")
  echo -e "  Last modified: $last_mod"
  echo "--------------------------------------"
done
