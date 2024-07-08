FROM debian:bullseye-slim as build

RUN apt-get update && apt-get install -y curl build-essential libssl-dev pkg-config

WORKDIR /app

ENV RUSTUP_HOME=/home/app_user/.rustup \
    RUSTFLAGS="-C target-feature=-crt-static" \
    CARGO_HOME=/home/app_user/.cargo  \
    CARGO_TARGET_DIR=/home/app_user/target \
    PATH="/home/app_user/.cargo/bin:$PATH"

RUN curl --proto  '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.75 && rustup default 1.75

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY src /app/src
COPY regexes /app/regexes

COPY test_each_file/Cargo.toml /app/test_each_file/Cargo.toml
COPY test_each_file/src /app/test_each_file/src
COPY test_each_file/src /app/test_each_file/src

VOLUME ["/app"]

WORKDIR /app/

RUN cargo build --release --features full

FROM debian:bullseye-slim as run

RUN groupadd app_group && \
    useradd -g app_group app_user && \
    mkdir -p /home/app_user && \
    chown -R app_user:app_group /home/app_user

EXPOSE 8080

COPY --from=build /home/app_user/target/release/rust-device-detector /usr/local/bin/rust-device-detector
COPY --from=build /home/app_user/target/release/librust_device_detector.so /usr/local/lib/librust_device_detector.so
COPY --from=build /home/app_user/target/release/librust_device_detector.a /usr/local/lib/librust_device_detector.a

USER app_user:app_group
WORKDIR /app/

CMD ["/usr/local/bin/rust-device-detector", "-s", "-l", "0.0.0.0", "-c", "2000"]
