# TSLM
## Terminal Stream Last Mile

 This module is intended as a last mile node acting as gateway at the network boundaries. 
 
* Adapts internal message format to public or 3rd party protocols.
* Receives connections from the internal network.
* Receives connections from 3rd party or public networks.
  
 As a security measure connections that are originated from 3rd party or public networks are not allowed to connect 
inward into the internal network. Since the TSLM nodes sit at the boundaries of the network they accept connections from
internet and from the internal network but do not establish connections to other nodes.
 
 The terminal stream internal network is aware of the TSLM nodes and connects to them instead.

# TSLM node

 The server backend accepting ts streams and accepting websocket connections.

# TSLM lib

 A typescript library that connects to the TSLM websocket endpoint.