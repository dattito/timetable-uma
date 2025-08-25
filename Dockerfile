FROM cgr.dev/chainguard/rust:latest-dev AS build
WORKDIR /app
COPY . .
USER root
RUN apk update && apk add libssl3 openssl-dev libcrypto3
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && cp /app/target/release/timetable-uma ./myapp


FROM cgr.dev/chainguard/glibc-dynamic
COPY --from=build /usr/lib/libssl.so.3 /usr/lib/libssl.so.3
COPY --from=build /usr/lib/libcrypto.so.3 /usr/lib/libcrypto.so.3 
COPY --from=build --chown=nonroot:nonroot /app/myapp /usr/local/bin/myapp
ENV IN_DOCKER=true
CMD ["/usr/local/bin/myapp"]
