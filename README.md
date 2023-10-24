# TSLM - Terminal Stream Last Mile

 This is an async websocket server written in Rust using tokio-tungstenite.

 Allows for a websocket client to:
    * Create a channel.
    * Notify the channel with some data.
    * Subscribe to a channel.
    * Receive messages from the subscribed channels.

 My reason to write this is that I have built a service that streams real time technical indicator data and 
events for trading and I wanted to display some of the data online using TradingView's charts with a few considerations:

 1) The streams use gRPC/Protobuf and I wanted to display the information on a public website
 2) Easiest way to stream data online to a website is using websockets
 3) I don't want internet/public traffic connecting to my internal/private network

 So this server is supposed to sit in the boundaries of the network where it binds to a specific Ip/Port for public
connections and another one for the internal traffic pushing the data I want to stream to the public.

 This is a server I have put together over the weekend, don't expect much!
 

 