# syntax=docker/dockerfile:1.7
# reqx Development Environment
FROM rust:1.83-bookworm AS dev

ARG USERNAME=developer
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# Dependências de desenvolvimento
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget \
    vim \
    ripgrep \
    fd-find \
    jq \
    gdb \
    && rm -rf /var/lib/apt/lists/*

# Ferramentas Rust
RUN rustup component add \
    rustfmt \
    clippy \
    rust-analyzer \
    llvm-tools-preview

# Ferramentas cargo adicionais
RUN cargo install --locked \
    cargo-watch \
    cargo-nextest \
    cargo-audit \
    cargo-outdated \
    cargo-deny \
    cargo-llvm-cov \
    cargo-edit \
    just

# Usuário não-root
RUN groupadd --gid ${USER_GID} ${USERNAME} \
    && useradd --uid ${USER_UID} --gid ${USER_GID} -m ${USERNAME} \
    && mkdir -p /home/${USERNAME}/.cargo \
    && chown -R ${USERNAME}:${USERNAME} /home/${USERNAME}

ENV CARGO_HOME=/home/${USERNAME}/.cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"

USER ${USERNAME}
WORKDIR /workspace

CMD ["bash"]
