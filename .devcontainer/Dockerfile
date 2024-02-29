FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm
# FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
  && apt-get -y install --no-install-recommends \
  clang \
  cmake \
  llvm \
  libeditorconfig-dev \
  && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

USER vscode
RUN cargo install bindgen-cli
