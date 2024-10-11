# Bevygap Spaceships

A deployable, production ready demo game using Lightyear and Edgegap.

A learning exercise / started kit for deploying your own game.


## Building the server

To run during development you can of course:
```
cargo run -p server
```

To build a docker container locally, you can say:
```
docker build -f server/Dockerfile .
```

Or to trigger the github action to build the server docker image and push to the Edgegap container registry, ready for deployment, you can either push a new git tag in the format `v1.2.3` or manually trigger the workflow from the github actions page of the repository.