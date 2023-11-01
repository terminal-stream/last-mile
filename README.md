# TSLM - Terminal Stream Last Mile

---

This is an asynchronous WebSocket server written in Rust using tokio-tungstenite. It allows a WebSocket client to
perform the following actions:

- Create a channel.
- Notify the channel with relevant data.
- Subscribe to a channel.
- Receive messages from the subscribed channels.

The inception of this server finds its roots in the creation of a service for a companion project, "terminal.stream."

Terminal Stream is the source of real-time technical indicators and trading events. My objective is to thoughtfully
curate and present select events to the public, rather than exposing all of them.

Our aim is to effectively showcase a subset of this data on a public website, all while preserving the confidentiality
of our internal services. Several considerations drive this endeavor:

1. The technical indicator streams are delivered through gRPC/Protobuf, presenting challenges in directly pushing this
   data to a web interface.
2. The most efficient method for real-time data display on a website is through WebSocket connections.
3. There's a pressing requirement to demarcate and shield our internal/private network from the internet/public traffic.

Consequently, this server serves as an intermediary within the network, binding to specific IP addresses and ports. It
allows public connections while also facilitating the secure transmission of the desired data to the public domain.

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

## TODO

* ~~Server configuration.~~
* ~~Multiple network interface/ip/port binding.~~
* Client fallback mechanism by protocol.
* Channel backlog.
* Identified messages for replies.
* Auth with 3rd party sso using oauth2.
* Using white listed public keys and hashes to validate publishers.
* Channel configuration.
* Docker to make it easier for others to build and run locally and give the server a try.