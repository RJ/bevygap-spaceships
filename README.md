# Bevygap Spaceships

A demo multiplayer game using Bevy and Lightyear, ready to deploy to Edgegap's infrastructure to automatically scale server deployment and matchmake..

This uses my (work-in-progress) [Bevygap](https://github.com/RJ/bevygap) project.

A learning exercise / starter kit / tutorial.

Find me on the bevy discord as @RJ

# Live Demo

Is the [live demo of bevygap-spaceships](https://game.metabrew.com/bevygap-spaceships/) working? Hopefully!

## Running Without Bevygap

To run server and client during development, without the connect token stuff, edgegap, or matchmaker from bevygap, disable the default bevygap feature:

```
cargo run --no-default-features -p server
# and:
cargo run --no-default-features -p client
```

No need to set the `LIGHTYEAR_PRIVATE_KEY`, it'll use all zeros, which is insecure but fine for local development.

## Running with Edgegap in the loop

Follow the setup instructions in the [Bevygap book](https://rj.github.io/bevygap/).

Once you can deploy containerized game servers to edgegap that successfully startup and connect to your NATS instance, you can run the rest of the components locally, as long as they connect to your NATS.

#### Run the matchmaker

In the [bevygap](https://github.com/RJ/bevygap) dir (separate repo), run the matchmaker:
```
# make sure your NATS_USER etc envs are set!
cargo run -p bevygap_matchmaker -- --app-name bevygap-spaceships --app-version 1 --lightyear-protocol-id 80085  --lightyear-private-key '1,2, 3, ... 0'
```

and the matchmaker webservice, which listens on `:3000`:

```
cargo run -p bevygap_matchmaker_httpd
```

And then from this repo, run the client, setting the matchmaker URL to your local machine:

```
MATCHMAKER_URL=ws://127.0.0.1:3000/matchmaker/ws cargo run -p client
```

When you click connect in the client, your local matchmaker will end up talking to Edgegap, spawning or selecting a server, and you should find yourself connected to a gameserver running in Edgegap's cloud.

## Remember..

Ensure your envs are set.

Remember the `LIGHTYEAR_PRIVATE_KEY` the server uses must match the key given to the matchmaker for connect tokens to work. You can copy from the rust source and set like this, non alphanumeric chars will be ignored. This would need to be set in Edgegap where the servers are deployed.

```
LIGHTYEAR_PRIVATE_KEY="[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]"
```



## ENV

Attempt to document envs used..

| Environment Variable           | Default                                                                                                      | Description                                                                            |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- |
| `LIGHTYEAR_CERTIFICATE_DIGEST` | Empty string                                                                                                 | Only needed if testing wasm clients without bevygap, which sets this for you           |
| `LIGHTYEAR_PRIVATE_KEY`        | Zeroed key                                                                                                   | Required when using bevygap. Must match value in matchmaker for connect tokens to work |
| `MATCHMAKER_URL`               | <small>Native:&nbsp;`ws://localhost:3000/matchmaker/ws`<br>Wasm:&nbsp;`ws(s)://{host}/matchmaker/ws`</small> | URL of the matchmaker service                                                          |

## Server Notes

To build a server docker container locally, you can say:
```
docker build -t bevygap-spaceships-server:latest -f server/Dockerfile .
```

I'm mostly using github actions to bake my containers. See github workflows for details.

## WASM Notes

The included `client/Dockerfile` builds the wasm and creates a container based on `nginx` which will serve up the `index.html` and wasm assets for you. 

If you don't want to use the docker file:

Use `clang` instead of `cc` for wasm compilation, to avoid weird linker issues on mac/linux.

`apt install clang` on linux or `brew install llvm` on mac.

Build wasm like this (fixing the path to your clang):
```
TARGET_CC=/opt/homebrew/opt/llvm/bin/clang \
RUSTFLAGS=--cfg=web_sys_unstable_apis \
cargo build --release --target wasm32-unknown-unknown -p client

wasm-bindgen \
    --no-typescript \
    --target web \
    --out-dir ./out \
    --out-name "bevygap-spaceships" \
    ./target/wasm32-unknown-unknown/release/client.wasm
```

You can `cd ./out && python3 -m http.server` to test locally.


><small>⚠️ If Mac Docker builds fail with SIGKILL, you may need to increase RAM for Docker's VM
</small>
