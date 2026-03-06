# qso-logger

## IPv6 tunnel mode

This project now supports an amateur-radio IPv6 tunnel mode using the
`2602:fa86:44::/48` prefix and unicast-prefix multicast groups:

- site-local scope: `ff35:30:2602:fa86:44::/96`
- global scope: `ff3e:30:2602:fa86:44::/96`

Run one side in server mode and the other in client mode:

```bash
# server: accepts HTTPS and WSS tunnel sessions
cargo run -- --server --bind [::]:8443 --upstream [::1]:9000 --cert cert.pem --key key.pem

# client: accepts local TCP and tunnels it to server WSS
cargo run -- --client --listen [::1]:7000 --server-url wss://localhost:8443/tunnel --insecure
```
