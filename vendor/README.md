# Vendored MySQL client

`mysql-28.0.3` comes from the published `mysql` 28.0.3 crate:

- Source: <https://static.crates.io/crates/mysql/mysql-28.0.3.crate>
- SHA256: `7d3d30e2806645e218163f49c2656286061cc91ffce24d93d758fd136e70eb5d`
- The registry's `.cargo_vcs_info.json` is omitted.

Only two source files carry local functionality changes:

- `src/conn/opts/mod.rs` adds `server_public_key_path` storage and accessors.
- `src/conn/mod.rs` uses that key for non-TLS `caching_sha2_password`
  authentication, retaining upstream key retrieval when none is configured.

`src/conn/query_result.rs` also retains a whitespace-only cleanup from the
previous vendored version.

PDO's `ATTR_SERVER_PUBLIC_KEY` needs this extension. Remove the patch and this
vendored crate once an upstream release provides equivalent support.

Cipher-suite selection uses upstream's unmodified `SslOpts::with_cipher_suites`.
PDO's OpenSSL/IANA name conversion lives in
`crates/elephc-pdo/src/my/tls.rs`.
