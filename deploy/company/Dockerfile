FROM rust:1.97-bookworm AS build
WORKDIR /src
COPY starter/ ./starter/
WORKDIR /src/starter
RUN cargo build --release -p api

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --create-home app
COPY --from=build /src/starter/target/release/starter-api /usr/local/bin/starter-api
USER 10001
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/starter-api"]
