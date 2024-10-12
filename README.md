# Bevygap Spaceships

A demo multiplayer game using Bevy and Lightyear, ready to deploy to Edgegap's infrastructure to automatically scale server deployment and matchmake..

A learning exercise / starter kit / tutorial.

Find me on the bevy discord as @RJ

## Building and Running

To runserver and client during development, without the connect token stuff from bevygap:
```
cargo run --no-default-features -p server
# and:
cargo run --no-default-features -p client
```

To build a server docker container locally, you can say:
```
docker build -t bevygap-spaceships-server:latest -f server/Dockerfile .
```

Or to trigger the github action to build the server docker image and push to the Edgegap container registry, ready for deployment, you can either push a new git tag in the format `v1.2.3` or manually trigger the workflow from the github actions page of the repository.


## NOTES

it's quicker to `cargo build` on the gh runner then copy build artefacts to the container, but there
isn't a linux with the same glibc version as the one in distroless, and i can't seem to cross compile the
udev stuff to musl, so need to build in a container too.. makes caching harder?

rustup target install wasm32-unknown-unknown

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --target wasm32-unknown-unknown -p client

