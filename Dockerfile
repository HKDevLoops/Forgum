# syntax=docker/dockerfile:1
# =============================================================================
# Forgum: Multi-Stage Standalone & Sandbox Container
# Provides a zero-modification testing environment with TrueColor terminal,
# pre-configured shells (Zsh, Bash, Fish, Tmux), and self-contained updates.
# =============================================================================

# --- Stage 1: Build from source ---
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy cargo manifest and dependency trees
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY data/ data/

# Build optimized release binary
RUN cargo build --release --locked -p forgum-engine --bin forgum

# --- Stage 2: Runtime Sandbox ---
FROM debian:bookworm-slim AS runtime

LABEL maintainer="HKDevLoops <hkdevloops@example.com>"
LABEL description="Zero-modification interactive sandbox container for Forgum"
LABEL license="MIT"

ENV DEBIAN_FRONTEND=noninteractive
ENV TERM=xterm-256color
ENV COLORTERM=truecolor
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

# Install runtime terminal shells and utilities
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    zsh \
    fish \
    tmux \
    curl \
    git \
    procps \
    ca-certificates \
    less \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged standard user 'forgum'
RUN useradd -m -s /bin/zsh -u 1000 forgum

# Install binary and create legacy compatibility symlink
COPY --from=builder /build/target/release/forgum /usr/local/bin/forgum
RUN chmod +x /usr/local/bin/forgum && \
    ln -sf /usr/local/bin/forgum /usr/local/bin/forgum-engine

# Copy mascot art and data assets
RUN mkdir -p /usr/local/share/forgum /home/forgum/.local/share/forgum /home/forgum/.config/forgum
COPY --from=builder /build/data/ /usr/local/share/forgum/
COPY --from=builder /build/data/ /home/forgum/.local/share/forgum/
COPY packaging/containers/configs/unix-darwin-config.toml /home/forgum/.config/forgum/config.toml

# Provide forgum-sandbox-update script to allow pulling and testing updates within container
RUN cat << 'EOF' > /usr/local/bin/forgum-sandbox-update
#!/bin/bash
set -e
echo "Checking for Forgum updates..."
if command -v forgum >/dev/null 2>&1; then
    forgum update --check
fi
EOF
RUN chmod +x /usr/local/bin/forgum-sandbox-update

# Configure shell hooks for Zsh, Bash, and Fish
RUN echo '# Forgum Zsh Hook' >> /home/forgum/.zshrc && \
    echo 'eval "$(forgum init zsh)"' >> /home/forgum/.zshrc && \
    echo 'alias cowsay="forgum say"' >> /home/forgum/.zshrc && \
    echo 'alias lolcat="forgum render"' >> /home/forgum/.zshrc && \
    \
    echo '# Forgum Bash Hook' >> /home/forgum/.bashrc && \
    echo 'eval "$(forgum init bash)"' >> /home/forgum/.bashrc && \
    echo 'alias cowsay="forgum say"' >> /home/forgum/.bashrc && \
    echo 'alias lolcat="forgum render"' >> /home/forgum/.bashrc && \
    \
    mkdir -p /home/forgum/.config/fish && \
    echo '# Forgum Fish Hook' >> /home/forgum/.config/fish/config.fish && \
    echo 'forgum init fish | source' >> /home/forgum/.config/fish/config.fish

# Ensure proper permissions
RUN chown -R forgum:forgum /home/forgum

# Copy shell verification test suite
COPY packaging/containers/scripts/test-all-shells.sh /usr/local/bin/test-all-shells.sh
RUN chmod +x /usr/local/bin/test-all-shells.sh

USER forgum
WORKDIR /home/forgum

# Entry command launches interactive login shell
CMD ["/bin/zsh", "-l"]
