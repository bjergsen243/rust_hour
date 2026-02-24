FROM rust:1.84-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p rust-hour-alert

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/alert-service /usr/local/bin/
EXPOSE 50053
CMD ["alert-service"]
