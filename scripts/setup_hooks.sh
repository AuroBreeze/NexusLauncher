#!/bin/bash

# scripts/setup_hooks.sh
# Sets up the pre-commit framework for the project

echo "Setting up pre-commit framework..."

# 1. Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "pre-commit could not be found."
    echo "Attempting to install via pip..."
    if command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v pip &> /dev/null; then
        pip install pre-commit
    else
        echo -e "\033[0;31mError: pip is not installed. Please install Python and pip, then run 'pip install pre-commit'.\033[0m"
        exit 1
    fi
fi

# 2. Remove the old manual hook if it exists
if [ -f ".git/hooks/pre-commit" ]; then
    echo "Removing old manual pre-commit hook..."
    rm ".git/hooks/pre-commit"
fi

# 3. Install the pre-commit hooks defined in .pre-commit-config.yaml
echo "Installing pre-commit hooks..."
pre-commit install
pre-commit install --hook-type commit-msg

echo -e "\033[0;32mSuccess! pre-commit hooks are now active.\033[0m"
echo "You can manually run all checks on all files using: pre-commit run --all-files"
