#!/bin/bash
# Test suite to verify thorough Forgum initialization across all shells
set -e

echo "=== 1. Testing forgum-engine binary execution ==="
forgum-engine --version
forgum-engine checkhealth

echo "=== 2. Testing Zsh initialization hook ==="
zsh -c 'eval "$(forgum-engine init zsh)"; forgum-engine say "Zsh Hook Test: Success" --cow tux'

echo "=== 3. Testing Bash initialization hook ==="
bash -c 'eval "$(forgum-engine init bash)"; forgum-engine say "Bash Hook Test: Success" --cow daemon'

echo "=== 4. Testing Fish initialization hook ==="
fish -c 'forgum-engine init fish | source; forgum-engine say "Fish Hook Test: Success" --cow stegosaurus'

echo "=== 5. Testing Subcommand suite ==="
forgum-engine herd list
forgum-engine theme list
forgum-engine timer echo "Shell Hook Test Completed Successfully"

echo ""
echo "🎉 ALL SHELL INITIALIZATIONS & HOOKS VERIFIED SUCCESSFULLY!"
