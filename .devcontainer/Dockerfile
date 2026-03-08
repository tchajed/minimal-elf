FROM debian:trixie

ENV CC=x86_64-linux-gnu-gcc
ENV STRIP=x86_64-linux-gnu-strip
ENV OBJDUMP=x86_64-linux-gnu-objdump
ENV OBJCOPY=x86_64-linux-gnu-objcopy

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    crossbuild-essential-amd64 \
    qemu-user-static \
    file \
    strace \
    make \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup for the default user
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default
