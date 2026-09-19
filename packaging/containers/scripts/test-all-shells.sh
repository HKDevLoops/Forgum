#!/bin/bash
# Test suite to verify thorough Forgum initialization across all shells
set -e

echo "=== 1. Testing forgum binary execution ==="
forgum --version
forgum doctor
forgum checkhealth

echo "=== 2. Testing Zsh initialization hook ==="
zsh -c 'eval "$(forgum init zsh)"; forgum --cow tux render "Zsh Hook Test: Success"'

echo "=== 3. Testing Bash initialization hook ==="
bash -c 'eval "$(forgum init bash)"; forgum --cow daemon render "Bash Hook Test: Success"'

echo "=== 4. Testing Fish initialization hook ==="
fish -c 'forgum init fish | source; forgum --cow stegosaurus render "Fish Hook Test: Success"'

echo "=== 5. Testing Subcommand suite ==="
forgum herd list
forgum theme list
forgum list animals
forgum timer echo "Shell Hook Test Completed Successfully"

echo ""
echo "🎉 ALL SHELL INITIALIZATIONS & HOOKS VERIFIED SUCCESSFULLY!"
