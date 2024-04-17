# TSLM - Terminal Stream Last Mile

---

This is an asynchronous WebSocket server written in Rust using tokio-tungstenite. It allows a WebSocket client to
perform the following actions:

- Create a channel.
- Notify the channel with relevant data.
- Subscribe to a channel.
- Receive messages from the subscribed channels.

The purpose of this server is to act as a websocket gateway.

As a gateway publishers should be able to selectively push messages from the internal/private/intranet applications to
the gateway for it to fan out messages to subscribers connected from public networks without compromising security.

In the probable event of getting hacked the gateway server should not be able to connect inward into the intranet
limiting the area vulnerable to attacks.

---

## IMPORTANT NOTE:

This server is a weekend project, and while functional, it might not have the full range of features you might expect
from a more extensive, dedicated development effort.

---

## Acknowledgements

* The current message schemas are experimental and terrible JSON. The reason is that this initial implementation is
  serializing enums directly as it is quite practical for experimentation but terrible as API definitions.
  Message schemas will change drastically in the next versions.

---

## How to run

### From source

This is the source code so it is possible to just build and run locally running:

    cargo run

And get a local server running just to play around and debug.

Note: ***The code expects your local to have openssl installed.***

### As docker container

The repo also provides example dockerfiles at ./docker that can be used to build a docker image that you can push to
your
registry and run.

#### Build the dependencies

There are 3 dockerfiles at ./docker:

* 'build-amazonlinux-openssl-rust.dockerfile' that builds a 'builder' image.
* 'exec-amazonlinux-openssl.dockerfile' that build the execution environment image.
* 'last-mile.dockerfile' uses these 2 previously images as builder and execution environments.

So first go into ./docker and build the 2 images:

    # from ./docker
    ./docker_build.sh
    ./docker_exec.sh

#### Build the image

Now you can run ./docker_build.sh that will use the 'last-mile.dockerfile' to build the final image.

#### Run the image

'./docker-run.sh' is an example, just mounts ./config to a path where the container expects to find the config and runs
the server.

#### Conclusion

The docker builder image takes care of installing rust, openssl and other dependencies the project might require to
build.

Then the docker exec image continues by setting up the environment again but only with the minimal required binary
dependencies
and the required executables to run the server so the image should be as small as possible.

These things combined make the build process faster as it caches and reduces the final image size.

The examples are cross compiling to x86_64 which is the architecture I am using now at amazon, just change the
target architecture to the one you are running in the ./docker/last-mile.dockerfile instructions.

---

## TODO

* ~~Server configuration.~~
* ~~Multiple network interface/ip/port binding.~~
* Client fallback mechanism by protocol.
* Channel backlog.
* Identified messages for replies.
* Auth with 3rd party sso using oauth2.
* Using white listed public keys and hashes to validate publishers.
* Channel configuration.
* ~~Docker to make it easier for others to build and run locally and give the server a try.~~
