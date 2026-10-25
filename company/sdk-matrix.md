# SDK matrix

Keep the languages you already have. Rewrite only the data-owning core in Rust.

| Service/client | Language | Package | Version | Transport | Conformance | Keep/Rewrite |
|---|---|---|---|---|---|---|
| example-api | TypeScript | `@loams/client` | 0.1.1 | Connect | app_collections | keep |
| example-worker | Rust | `loams` | 0.1.1 | Connect | app_collections, app_streams | rewrite |

## Notes

- The SDKs are 0.x; pin the version.
- All languages share the runtime contract; do not write a per-language spec.
- A language without a required app module is a reason to wait or bridge.
