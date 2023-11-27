FROM messense/rust-musl-cross:x86_64-musl as builder
RUN apt update && apt install -y --no-install-recommends perl librust-openssl-dev libssl-dev
COPY . /home/rust/src
RUN rm /home/rust/src/Cargo.lock
RUN cargo search
ENV OPENSSL_DIR=/usr
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
RUN \
  --mount=type=cache,target=/home/rust/src/target,rw \
  --mount=type=cache,target=/usr/local/cargo/registry,rw
RUN cd /home/rust/src && \
  cargo build --target x86_64-unknown-linux-musl --release

FROM scratch

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/combine_images /
COPY --from=builder /home/rust/src/images /images
CMD ["/combine_images"]