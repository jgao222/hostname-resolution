# Hostname Resolution Server
Hostname resolution through HTTP. Currently just a glorified in-memory key/value store.

Intended for use in resolving custom hostnames, for example: clients finding host IPs in P2P applications based on a hostname.

Ideally, hosts send hashed IPs to associate with their names, and clients need to know a password (symmetric key hashing) to get the original IP address after requesting the value from the server.

## Spec
### POST
- POST with `Content-Type: application/x-www-form-urlencoded` and body params of `hostname=[YOUR_NAME]` and `host_value=[YOUR_VALUE]` to create a record on the server matching the provided hostname to the provided value.
- Returns `200` if record was created on the server, `400` if the request was bad.

### GET
- GET to any endpoint (currently ignored) with 1 query param of `?hostname=[TARGET_NAME]`.
- Returns `200` if the hostname is matched to a value on the server, `404` if not found, `400` if the request was bad.