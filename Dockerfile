FROM debian:bullseye-slim
WORKDIR /app
ADD target/debug/wager .
CMD ["/app/wager"]