#!/bin/bash

# scripts/test.sh
# Standalone test runner — split into non-network (parallel) and network (serial) phases.

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Running unit tests (non-network, parallel)...${NC}"
cargo test --workspace --exclude nexus-mods --exclude nexus-version

echo -e "\n${GREEN}Running network tests (serial)...${NC}"
cargo test -p nexus-mods -p nexus-version -- --test-threads=1

echo -e "\n${GREEN}All tests passed.${NC}"
