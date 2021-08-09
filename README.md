# replicate-rustls-bug

This example replicates the session resumption bug in `rustls` which occurs when the peer switches from TLS 1.3 to TLS 1.2.

1. Start `go-server` with max tls version 1.3 and `rustls-client`
```
~/rustls-bug/go-server$ go run main.go --conf data/config.yml --tls-version 1.3
~/rustls-bug/rustls-client$ RUST_LOG=trace cargo run data/config.yml
```
2. After `rustls-client` has sent a message (it sends one message every 15 seconds), restart `go-server` with max tls version 1.2 as 
`go run main.go --conf data/server.yml --tls-version 1.2` before the next 15 seconds elapse.
3. You should be able to see a varied ciphersuite error that looks like 
```
[2021-08-03T23:05:34Z ERROR rustls_client] failed to execute request with err: Custom { kind: InvalidData, error: PeerMisbehavedError("abbreviated handshake offered, but with varied cs") }
```
after `rustls-client` attempts to send a message.
