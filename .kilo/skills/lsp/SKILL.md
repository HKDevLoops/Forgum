---
name: lsp
description: LSP (Language Server Protocol) management and configuration skill
---

# LSP Management Skill

This skill provides guidance for managing Language Server Protocol (LSP) configurations in the project.

## Available LSP Servers

The project is configured with the following LSP servers:

### Rust
- **Server**: `rust-analyzer`
- **Extensions**: `.rs`
- **Installation**: Via rustup (`rustup component add rust-analyzer`)

### YAML
- **Server**: `yaml-language-server`
- **Extensions**: `.yaml`, `.yml`
- **Installation**: `npm install -g yaml-language-server`

### Markdown
- **Server**: `vscode-markdown-language-server`
- **Extensions**: `.md`, `.markdown`
- **Installation**: `npm install -g vscode-langservers-extracted`

### JSON
- **Server**: `vscode-json-language-server`
- **Extensions**: `.json`, `.jsonc`
- **Installation**: `npm install -g vscode-langservers-extracted`

### Bash/Shell
- **Server**: `bash-language-server`
- **Extensions**: `.sh`, `.bash`
- **Installation**: `npm install -g bash-language-server`

### CSS
- **Server**: `vscode-css-language-server`
- **Extensions**: `.css`, `.scss`, `.less`
- **Installation**: `npm install -g vscode-langservers-extracted`

### HTML
- **Server**: `vscode-html-language-server`
- **Extensions**: `.html`, `.htm`
- **Installation**: `npm install -g vscode-langservers-extracted`

### TypeScript/JavaScript
- **Server**: `typescript-language-server`
- **Extensions**: `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`
- **Installation**: `npm install -g typescript-language-server`

### ESLint
- **Server**: `vscode-eslint-language-server`
- **Extensions**: `.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs`
- **Installation**: `npm install -g vscode-langservers-extracted`

## LSP Configuration

The LSP configuration is stored in `kilo.json` under the `lsp` key. Each LSP server configuration includes:

- `command`: Array of command arguments to start the LSP server
- `extensions`: Array of file extensions that the LSP server handles
- `env`: Object of environment variables to pass to the LSP server

## Verifying LSP Installation

To verify that an LSP server is installed and working:

1. Check if the command is available in PATH
2. Run the command with `--version` flag
3. Test with a sample file of the appropriate extension

## Troubleshooting

### Common Issues

1. **LSP server not found**: Ensure the LSP server is installed globally via npm or cargo
2. **LSP server not starting**: Check if the command is correct in `kilo.json`
3. **LSP server not responding**: Check if the LSP server process is running

### Reinstalling LSP Servers

If an LSP server is not working correctly:

```bash
# For npm-based LSP servers
npm uninstall -g <package-name>
npm install -g <package-name>

# For cargo-based LSP servers
cargo uninstall <package-name>
cargo install <package-name>
```

## Adding New LSP Servers

To add a new LSP server:

1. Install the LSP server globally
2. Add the configuration to `kilo.json` under the `lsp` key
3. Test the LSP server with a sample file

## Production Testing

For production testing, use the `production-testing` skill which provides guidance on testing LSP servers in isolated environments using Docker, Podman, or WSL.
