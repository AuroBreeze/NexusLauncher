#!/bin/bash

# Do NOT use set -e, so we can run all checks even if some fail
# set -e

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

FAILED=0

echo -e "${GREEN}Starting pre-push checks...${NC}"

# 1. Check formatting
echo -e "\n${GREEN}[1/5] Checking code formatting (cargo fmt)...${NC}"
if ! cargo fmt --all -- --check; then
  echo -e "${RED}Formatting check failed!${NC}"
  FAILED=1
fi

# 2. Run Clippy (Lint)
echo -e "\n${GREEN}[2/5] Running code linting (cargo clippy)...${NC}"
if ! cargo clippy --all-targets --all-features -- -D warnings; then
  echo -e "${RED}Clippy linting failed!${NC}"
  FAILED=1
fi

# 3. Run Tests
echo -e "\n${GREEN}[3/5] Running automated tests (cargo test)...${NC}"
if ! cargo test --all-targets --all-features; then
  echo -e "${RED}Automated tests failed!${NC}"
  FAILED=1
fi

# 4. Build Project
echo -e "\n${GREEN}[4/5] Building project (cargo build)...${NC}"
if ! cargo build --release; then
  echo -e "${RED}Project build failed!${NC}"
  FAILED=1
fi

# 5. Check Commit Messages
echo -e "\n${GREEN}[5/5] Checking commit messages...${NC}"
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
