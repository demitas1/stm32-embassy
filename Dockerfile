FROM rust:1.92-bookworm

ENV DEBIAN_FRONTEND=noninteractive

# Build tools and dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libusb-1.0-0-dev \
    libudev-dev \
    gdb-multiarch \
    openocd \
    && rm -rf /var/lib/apt/lists/*

# Rust target for Cortex-M4F (STM32F4)
RUN rustup target add thumbv7em-none-eabihf
RUN rustup component add rust-src rust-analyzer

# Embedded Rust tools
RUN cargo install probe-rs-tools --locked
RUN cargo install flip-link --locked

# Create developer user
ARG UID=1000
ARG GID=1000
RUN groupadd -g ${GID} developer && \
    useradd -m -u ${UID} -g ${GID} -G dialout,plugdev developer

# Copy cargo and rustup to developer's home
RUN cp -r /usr/local/cargo /home/developer/.cargo && \
    cp -r /usr/local/rustup /home/developer/.rustup && \
    chown -R developer:developer /home/developer/.cargo /home/developer/.rustup

USER developer
ENV CARGO_HOME=/home/developer/.cargo
ENV RUSTUP_HOME=/home/developer/.rustup
ENV PATH="${CARGO_HOME}/bin:${PATH}"

WORKDIR /projects

CMD ["/bin/bash"]
