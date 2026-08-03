---
name: production-testing
description: Production testing using Docker, Podman, WSL, and sandbox environments
---

# Production Testing Skill

This skill provides guidance for testing the application and LSP servers in production-like environments using Docker, Podman, WSL, or sandbox environments.

## Available Tools

### Docker
- **Installation**: Docker Desktop for Windows
- **Path**: `C:\Users\haris\AppData\Local\Programs\DockerDesktop\resources\bin\docker.exe`
- **Usage**: Container-based testing

### Podman
- **Installation**: Podman for Windows
- **Path**: `C:\Users\haris\AppData\Local\Programs\Podman\podman.exe`
- **Usage**: Daemonless container engine

### WSL (Windows Subsystem for Linux)
- **Installation**: WSL enabled on Windows
- **Path**: `C:\WINDOWS\system32\wsl.exe`
- **Usage**: Linux environment on Windows

## Testing Strategies

### 1. Docker/Podman Container Testing

Create a Dockerfile for production testing:

```dockerfile
FROM rust:latest

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install LSP servers
RUN npm install -g \
    yaml-language-server \
    vscode-langservers-extracted \
    bash-language-server \
    typescript-language-server

# Copy project files
WORKDIR /app
COPY . .

# Run tests
CMD ["cargo", "test"]
```

Build and run the container:

```bash
# Using Docker
docker build -t forgum-test .
docker run --rm forgum-test

# Using Podman
podman build -t forgum-test .
podman run --rm forgum-test
```

### 2. WSL Testing

Test in a Linux environment using WSL:

```bash
# Enter WSL
wsl

# Install dependencies
sudo apt update
sudo apt install -y curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install LSP servers
npm install -g \
    yaml-language-server \
    vscode-langservers-extracted \
    bash-language-server \
    typescript-language-server

# Run tests
cargo test
```

### 3. Sandbox Testing

Use Windows Sandbox for isolated testing:

1. Enable Windows Sandbox feature
2. Create a sandbox configuration file
3. Mount the project directory
4. Install dependencies and run tests

## LSP Testing in Production

### Verify LSP Servers in Container

```bash
# Check if LSP servers are installed
docker run --rm forgum-test bash -c "which rust-analyzer yaml-language-server vscode-json-language-server bash-language-server"

# Test LSP server startup
docker run --rm forgum-test bash -c "echo '{}' | vscode-json-language-server --stdio"
```

### Test LSP with Sample Files

Create test files for each LSP server:

```bash
# Test YAML LSP
echo "key: value" > test.yaml
docker run --rm -v "$(pwd):/app" forgum-test bash -c "yaml-language-server --stdio < /app/test.yaml"

# Test JSON LSP
echo '{"key": "value"}' > test.json
docker run --rm -v "$(pwd):/app" forgum-test bash -c "vscode-json-language-server --stdio < /app/test.json"
```

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: Production Testing

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '20'
    
    - name: Install LSP servers
      run: |
        npm install -g \
          yaml-language-server \
          vscode-langservers-extracted \
          bash-language-server \
          typescript-language-server
    
    - name: Run tests
      run: cargo test
    
    - name: Verify LSP servers
      run: |
        rust-analyzer --version
        yaml-language-server --version
        vscode-json-language-server --version
        bash-language-server --version
        typescript-language-server --version
```

## Troubleshooting

### Common Issues

1. **Docker/Podman not starting**: Ensure Docker Desktop or Podman is running
2. **WSL not available**: Enable WSL feature in Windows Features
3. **Container build fails**: Check Dockerfile for syntax errors
4. **LSP servers not found in container**: Ensure they are installed in the Dockerfile

### Performance Tips

1. Use multi-stage builds to reduce image size
2. Cache npm dependencies in Docker layers
3. Use `.dockerignore` to exclude unnecessary files
4. Run tests in parallel when possible

## Security Considerations

1. Do not expose sensitive data in containers
2. Use non-root users in production containers
3. Scan images for vulnerabilities
4. Use read-only file systems where possible
