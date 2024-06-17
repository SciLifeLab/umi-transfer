FROM rust:bookworm as buildenv

WORKDIR /usr/app/src
COPY ./ /usr/app/src

RUN apt-get update && apt-get -y install clang cmake && \
    rm -rf /var/lib/apt/lists/* && \
    rustup component add rustfmt 

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/rust/target \
    cargo build --release

FROM debian:bookworm-slim as runner
WORKDIR /root
COPY --from=buildenv /usr/app/src/target/release/ /usr/local/bin/
RUN chmod 755 /usr/local/bin/umi-transfer
RUN useradd -s /bin/bash umi-transfer-user
USER umi-transfer-user

CMD [ "/bin/bash", "-l","-c"]
