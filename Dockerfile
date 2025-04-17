FROM rust:alpine AS builder
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM alpine:latest
COPY --from=builder target/release/crawl-ladder crawl-ladder
EXPOSE 3000
ENTRYPOINT ["./crawl-ladder"]
