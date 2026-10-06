FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev cmake make perl
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/static-debian12
COPY --from=build /app/target/release/perplexity-mcp /perplexity-mcp
ENV HOST=0.0.0.0
EXPOSE 8000
ENTRYPOINT ["/perplexity-mcp"]
