#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Color definitions
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting pre-push checks...${NC}"

# 1. Check formatting
echo -e "\n${GREEN}[1/5] Checking code formatting (cargo fmt)...${NC}"
cargo fmt --all -- --check

# 2. Run Clippy (Lint)
echo -e "\n${GREEN}[2/5] Running code linting (cargo clippy)...${NC}"
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run Tests
echo -e "\n${GREEN}[3/5] Running automated tests (cargo test)...${NC}"
cargo test --all-targets --all-features

# 4. Build Project
echo -e "\n${GREEN}[4/5] Building project (cargo build)...${NC}"
cargo build --release

# 5. Check Commit Messages
echo -e "\n${GREEN}[5/5] Checking commit messages...${NC}"
./scripts/check_commits.sh origin/main

echo -e "\n${GREEN}All checks passed successfully! It's safe to push.${NC}"
