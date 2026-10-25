![Loams — Your data. Your bucket.](../docs/assets/loams-banner.svg)

# Loams SDKs

This directory contains Loams language implementations, code generation templates, and conformance fixtures. Packages are pre-release and not a stable published SDK family. Check each language’s README and the conformance runner before relying on a runtime or transport.

## Language implementations

| Language | Path | Key Packages / Modules |
| --- | --- | --- |
| **TypeScript** | [`typescript/`](typescript/) | `@loams/client`, `@loams/live` |
| **Python** | [`python/`](python/) | `loams` |
| **Rust** | [`rust/`](rust/) | `loams-sdk` |
| **Go** | [`go/`](go/) | `github.com/ostrium-labs/loams/sdks/go` |
| **C# (.NET)** | [`csharp/`](csharp/) | `Loams` (app modules: namespaces, collections, documents, search, vector, graph) |
| **Java** | [`java/`](java/) | `dev.loams` (app modules: namespaces, collections, documents, search, vector, graph) |
| **Kotlin** | [`kotlin/`](kotlin/) | `dev.loams.kotlin` (app modules: namespaces, collections, documents, search, vector, graph) |
| **Swift** | [`swift/`](swift/) | `Loams` (app modules: namespaces, collections, documents, search, vector, graph) |
| **Ruby** | [`ruby/`](ruby/) | `loams` (app modules: namespaces, collections, documents, search, vector, graph) |
| **C++** | [`cpp/`](cpp/) | `loams` (app modules: namespaces, collections, documents, search, vector, graph) |
| **Dart** | [`dart/`](dart/) | `loams` (app modules: namespaces, collections, documents, search, vector, graph) |
| **Objective-C** | [`objc/`](objc/) | `LoamsObjC` (app modules: namespaces, collections, documents, search, vector, graph) |
| **PHP** | [`php/`](php/) | `loams/loams` (app modules: namespaces, collections, documents, search, vector, graph) |

## App modules (SDKP)

What each SDK's public client offers for building an app (`docs/plans/2026-10-19-sdk-parity.md`).
A row is added by that language's SDKP task; the app suite is
`node sdks/conformance/check-languages.mjs --check <lang> --suite app`.

| Language | namespaces · collections · documents · search · vector · graph | sql · streams | Builders, handle, R11–R14 | App suite |
| --- | --- | --- | --- | --- |
| TypeScript | yes | yes | yes | `app_collections`, `app_search`, `app_graph`, `app_sql`, `app_streams` |
| Python | yes, sync and async | yes | yes | `app_collections`, `app_search`, `app_graph`, `app_sql`, `app_streams` |
| Go | yes | yes | yes | `app_collections`, `app_search`, `app_graph`, `app_sql`, `app_streams` |
| Rust | yes | yes | yes | `app_collections`, `app_search`, `app_graph`, `app_sql`, `app_streams` |

## Installing

The SDKs are published as `0.1.1` from the Loams release tag by
[`.github/workflows/sdk-release.yml`](../.github/workflows/sdk-release.yml), to
npm, PyPI, crates.io, NuGet, the Go proxy, RubyGems and Maven Central; Swift
resolves from this repository's tag and C++ ships as a release tarball. PHP
(Packagist), Dart (pub.dev) and Objective-C (CocoaPods) are not on their
registries yet. In 0.x an API may change between minor versions.

| Language | Install |
| --- | --- |
| TypeScript | `npm install @loams/client` |
| Python | `pip install loams` |
| Rust | `cargo add loams` |
| Go (1.25+) | `go get loams.dev/go@v0.1.1` |
| C# | `dotnet add package Loams` |
| Swift | `.package(url: "https://github.com/ostrium-labs/loams.git", from: "0.1.1")` |
| C++ | `loams-cpp-0.1.1.tar.gz` on the GitHub release; `find_package(loams CONFIG)` |
| Ruby | `gem install loams` |
| PHP | `composer require loams/loams` (from the split repository; see [`php/`](php/)) |
| Dart | `dart pub add loams` |
| Java | `dev.loams:loams:0.1.1` (Maven Central) |
| Kotlin | `dev.loams:loams-kotlin:0.1.1` (Maven Central) |
| Objective-C | `pod 'Loams', '0.1.1'` (module `LoamsObjC`) |

Each registry goes live once the owner has set it up; which ones are live is in
[`docs/release/publishing.md`](../docs/release/publishing.md).

## Conformance & Testing

- **[`conformance/`](conformance/)**: Cross-language conformance test runner verifying that all SDKs adhere to the Loams runtime contract (idempotency, consistency tokens, error representations, and pagination).
- **[`fixtures/`](fixtures/)**: Golden test vectors and fixtures consumed by all SDK conformance test suites.
- **[`templates/`](templates/)**: Generator templates used by `scripts/sdk/gen.sh` to produce unified facades and typed clients.
