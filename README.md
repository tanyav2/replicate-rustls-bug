# replicate-rustls-bug

1. Run `go-server` with max tls version 1.3 as `~/rustls-bug/go-server$ go run main.go --conf data/config.yml --tls-version 1.3`
2. Then run the `rustls-client` with `RUST_LOG=trace` as `~/rustls-bug/rustls-client$ RUST_LOG=trace cargo run data/config.yml` 
3. After `rustls-client` has sent a message (it sends one message every 15 seconds), restart `go-server` with max tls version 1.2 as `go run main.go --conf data/server.yml --tls-version 1.2` before the next 15 seconds elapse.
4. You should be able to see a varied ciphersuite error that looks like 
```
[2021-08-03T23:05:34Z ERROR rustls_client] failed to execute request with err: Custom { kind: InvalidData, error: PeerMisbehavedError("abbreviated handshake offered, but with varied cs") }
```
after `rustls-client` attempts to send a message.
