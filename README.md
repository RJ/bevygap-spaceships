# Bevygap Spaceships

A demo multiplayer game using Bevy and Lightyear, ready to deploy to Edgegap's infrastructure to automatically scale server deployment and matchmake..

A learning exercise / starter kit / tutorial.

Find me on the bevy discord as @RJ

## Building the server

To run during development you can of course:
```
cargo run -p server
```

To build a docker container locally, you can say:
```
docker build -t bevygap-spaceships-server:latest -f server/Dockerfile .
```

Or to trigger the github action to build the server docker image and push to the Edgegap container registry, ready for deployment, you can either push a new git tag in the format `v1.2.3` or manually trigger the workflow from the github actions page of the repository.