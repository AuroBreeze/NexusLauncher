#!/bin/bash

# Do NOT use set -e, so we can run all checks even if some fail
# set -e

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

FAILED=0

echo -e "${GREEN}Starting pre-push checks...${NC}"

# 1. Auto-format code
echo -e "\n${GREEN}[1/4] Auto-formatting code (cargo fmt)...${NC}"
cargo fmt --all
echo -e "${GREEN}Formatting applied.${NC}"

# 2. Run Clippy (Lint)
echo -e "\n${GREEN}[2/4] Running code linting (cargo clippy)...${NC}"
if ! cargo clippy --all-targets --all-features -- -D warnings; then
  echo -e "${RED}Clippy linting failed!${NC}"
  FAILED=1
fi

# 3. Run Tests — skipped locally (too slow with network tests); CI runs them in full
# TODO: Add argument parsing (e.g. --test) to optionally run ./scripts/test.sh
# Manual:
#   ./scripts/test.sh

# 4. Check Commit Messages
echo -e "\n${GREEN}[4/4] Checking commit messages...${NC}"
if ! ./scripts/check_commits.sh origin/main; then
  echo -e "${RED}Commit message check failed!${NC}"
  FAILED=1
fi

echo -e "\n--------------------------------------"
if [ $FAILED -eq 1 ]; then
  echo -e "${RED}Some checks failed. Please fix the issues before pushing.${NC}"
  exit 1
else
  echo -e "${GREEN}All checks passed successfully! It's safe to push.${NC}"
  exit 0
fi
