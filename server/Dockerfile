# Since "cargo build" ran in the github action vm, we simply pick a very minimal
# container and copy the compiled binary and assets into it for distribution.
#
# the :debug version includes a busybox shell if you need it..
# FROM gcr.io/distroless/cc-debian12:debug
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY ./assets/ /app/assets/
COPY ./certificates/ /app/certificates/
COPY ./target/x86_64-unknown-linux-musl/release/server /app/

ENTRYPOINT ["./server"]

# documents but doesn't automatically expose:
EXPOSE 6420/udp
