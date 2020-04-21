FROM clux/muslrust:latest as builder
WORKDIR /app
ADD . .
RUN cargo build --release


FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/wager /app/wager
ENTRYPOINT ["/app/wager"]