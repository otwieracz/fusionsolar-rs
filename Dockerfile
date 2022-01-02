FROM alpine:3.15
ADD target/x86_64-unknown-linux-musl/release/fusionsolar-rs /
ENV ROCKET_ADDRESS "0.0.0.0"
ENTRYPOINT "/fusionsolar-rs"