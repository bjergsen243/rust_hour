FROM rust:1.84-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p rust-hour-monitor

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/monitor-service /usr/local/bin/
EXPOSE 50052
CMD ["monitor-service"]
