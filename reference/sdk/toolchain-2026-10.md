# SDK toolchain, October 2026

API2 Task 0 step 2 (SDK1 Task 0). One row per language of design [§44 §9](../design/44-unified-api-and-sdks.md#9-the-language-matrix-d612d614): its Connect or gRPC library, that library's latest release and date, the remote `buf` plugins and their latest versions, the registry, and whether the registry has trusted publishing. Checked on **2026-10-13** against `dev` at `b555fe73`. `sdks/conformance/check-languages.mjs --toolchain` runs `toolchain_table_has_row_per_language`, which fails when a language of `sdks/conformance/required.mjs`'s `LANGUAGES` has no row, has two, or has an empty cell.

This table records what is **published upstream**. It does not change a pin: the pins stay in `scripts/sdk/pins.lock`, in each `sdks/<lang>/buf.gen*.yaml` and in each `DEPENDENCIES.md` or lockfile. A version that is less than 14 days old on 2026-10-13 is not pinnable yet (the plan's pin rule), and the row says so.

## D612's 12-month rule

D612: a Connect library with no release in the last 12 months, or one that fails the conformance suite, falls back to gRPC. On 2026-10-13 the cutoff is **2025-10-13**.

- Every Connect library has released since the cutoff. The oldest latest release is `connect-swift` 1.2.3 (2026-05-21). **No language moves from Connect to gRPC.**
- **Dart changes column.** §44 §9 row 9 left it as "Connect if `connect-dart` is maintained and passes conformance, else gRPC". `connect-dart` released 2.0.0 on 2026-08-31, so the maintenance half is met and Dart is **Connect**. The conformance half is still open: the Dart SDK as built speaks the Connect wire through `package:http` and does not depend on `connectrpc` yet (`sdks/dart/pubspec.yaml`). It is decided when API2 Task 21 runs the suite with `connect-dart`. If that fails, Dart falls back to `grpc` (5.1.0, 2025-12-09, also inside the window).
- **C++ already changed column** before this check. D639 (SDK2 Task 11) makes it Connect and gRPC-Web over HTTP through `loams::HttpTransport`, with no grpc++. This table records that and does not reopen it.
- The rule covers Connect libraries only. For the record, every gRPC library in the table has also released inside the window.

## The table

| Language | Key | Protocol (2026-10-13) | Library | Latest release (date) | Remote plugins, latest version (date added to `bufbuild/plugins`) | Registry | Trusted publishing | Change from §44 §9 |
|---|---|---|---|---|---|---|---|---|
| TypeScript / JavaScript | `typescript` | Connect | `@connectrpc/connect` (connect-es) | 2.2.0 (2026-09-07) [npm][npm-connect] | `buf.build/bufbuild/es` v2.16.0 (2026-09-30) | npm | Yes, OIDC ([npm docs][tp-npm]) | None. Pinned at 2.2.0, which is the latest |
| Python | `python` | Connect | `connectrpc` (connect-py) | 0.12.1 (2026-09-04) [PyPI][pypi-connect] | `buf.build/protocolbuffers/python` v36.2 (2026-09-19); `buf.build/connectrpc/python` v0.10.1 (2026-06-25) | PyPI | Yes, OIDC ([PyPI docs][tp-pypi]) | None. The pins disagree: see "Pin drift" below |
| Go | `go` | Connect | `connectrpc.com/connect` (connect-go) | v1.21.0 (2026-09-08), the v1 line; v2.0.0 (2026-10-07) is not pinnable before 2026-10-21 [GitHub][gh-connect-go] | `buf.build/protocolbuffers/go` v1.36.12 (2026-08-11); `buf.build/connectrpc/go` v2.0.0 (2026-10-07), v1.21.0 for the v1 line | Go module proxy, through a tag on the mirror `ostrium-labs/loams-go` | Not applicable (git tag, no upload) | None |
| Rust | `rust` | Connect | `connectrpc` (connect-rust) with `buffa` | 0.9.1 (2026-09-21) [crates.io][crates-connect] | None. `connectrpc-build` runs in `build.rs` (§44 §9 row 4) | crates.io | Yes, OIDC ([Rust blog][tp-crates]) | None |
| Swift | `swift` | Connect | `connect-swift` | 1.2.3 (2026-05-21) [GitHub][gh-connect-swift] | `buf.build/apple/swift` v1.38.1 (2026-06-24); `buf.build/connectrpc/swift` v1.2.3 (2026-05-21) | SwiftPM, through a tag on the mirror `ostrium-labs/loams-swift` | Not applicable (git tag) | None. As built, the SDK has no package dependency and speaks Connect over `URLSession` (`sdks/swift/Package.swift`) |
| Kotlin (JVM, Android) | `kotlin` | Connect | `com.connectrpc:connect-kotlin` | 0.9.0 (2026-06-12) [Maven Central][mvn-connect-kotlin] | `buf.build/protocolbuffers/java` v36.2 (2026-09-19); `buf.build/connectrpc/kotlin` v0.9.0 (2026-06-25) | Maven Central | No: a Central Portal user token and a GPG key ([Sonatype][tp-maven]) | None. As built, the SDK depends on `protobuf-java` 4.35.1 only (`sdks/kotlin/build.sh`) |
| Java | `java` | gRPC | `io.grpc` (grpc-java) | 1.84.2 (2026-10-08, Maven Central metadata) [Maven Central][mvn-grpc-java]; the latest GitHub release is v1.84.1 (2026-10-07) | `buf.build/protocolbuffers/java` v36.2 (2026-09-19); `buf.build/grpc/java` v1.84.2 (2026-10-09) | Maven Central | No: a Central Portal user token and a GPG key ([Sonatype][tp-maven]) | None. As built, the SDK depends on `protobuf-java` 4.29.3 only (`sdks/java/build.sh`) |
| C# / .NET | `csharp` | gRPC | `Grpc.Net.Client` (grpc-dotnet) | 2.84.0 (2026-09-21) [NuGet][nuget-grpc] | `buf.build/protocolbuffers/csharp` v36.2 (2026-09-19); `buf.build/grpc/csharp` v1.84.0 (2026-09-14) | NuGet | Yes, OIDC ([Microsoft Learn][tp-nuget]) | None. As built, the SDK depends on `Google.Protobuf` 3.28.3 only (`sdks/csharp/src/Loams/Loams.csproj`) |
| Dart / Flutter | `dart` | Connect (gRPC if conformance fails) | `connectrpc` (connect-dart); fallback `grpc` | 2.0.0 (2026-08-31) [pub.dev][pub-connect]; `grpc` 5.1.0 (2025-12-09) [pub.dev][pub-grpc] | `buf.build/protocolbuffers/dart` v25.1.0 (2026-09-11); `buf.build/connectrpc/dart` v2.0.0 (2026-09-01) | pub.dev | Yes, OIDC ([dart.dev][tp-pub]) | **Changes column**: "Connect if maintained, else gRPC" becomes Connect. See D612 above |
| Ruby | `ruby` | gRPC, plus Connect-unary (D613) | `grpc` gem | 1.84.0 (2026-09-14) [RubyGems][gem-grpc] | `buf.build/protocolbuffers/ruby` v36.2 (2026-09-19); `buf.build/grpc/ruby` v1.84.0 (2026-09-14) | RubyGems | Yes, OIDC ([RubyGems guide][tp-rubygems]) | None |
| PHP | `php` | gRPC, plus Connect-unary (D613) | `ext-grpc` (PECL) and `grpc/grpc` (Composer) | PECL `grpc` 1.84.0 (2026-09-14) [PECL][pecl-grpc]; Packagist `grpc/grpc` 1.82.0 (2026-07-03) [Packagist][packagist-grpc] | `buf.build/protocolbuffers/php` v36.2 (2026-09-19); `buf.build/grpc/php` v1.84.0 (2026-09-14) | Packagist, through a webhook from the mirror `ostrium-labs/loams-php` | Not applicable (Packagist reads tags; nothing is uploaded) | None. The Composer package lags the extension by two minor versions |
| C++ | `cpp` | Connect and gRPC-Web over HTTP (D639) | none: `loams::HttpTransport` over libcurl 8.22.0 (`sdks/cpp/DEPENDENCIES.md`) | Not applicable. For reference, gRPC C++ is at 1.84.0 (2026-09-11) [GitHub][gh-grpc] | `buf.build/protocolbuffers/cpp` v36.2 (2026-09-19; the template pins v36.1); `buf.build/grpc/cpp` v1.84.0 (2026-09-14) is not used (D639) | vcpkg git registry and an in-repo Conan recipe (Q610) | No (no hosted upload; `SDK_PUBLISH_CONAN` gates a Conan remote) | Already changed by D639; unchanged here |
| Objective-C | `objc` | gRPC | gRPC-ObjC (`gRPC-ProtoRPC`) | 1.84.0 (2026-09-14, CocoaPods trunk) [CocoaPods][pod-grpc] | `buf.build/protocolbuffers/objc` v36.2 (2026-09-19); `buf.build/grpc/objc` v1.84.0 (2026-09-14) | Source and a SwiftPM target; no CocoaPods trunk publish (D614: trunk is read-only from 2026-12-02) | Not applicable | None. As built, the SwiftPM package has no dependency (`sdks/objc/Package.swift`) |

## Pin drift found while filling the table

These are recorded here and go to Task 13/17 (`scripts/sdk/check-pins.sh` and `drift.sh`). This change does not fix them, because a pin bump is its own PR with the regenerated output.

- **Python `connectrpc/python`.** `scripts/sdk/pins.lock` records `bsr buf.build/connectrpc/python v0.12.1` ("resolved by an actual `buf generate`" on 2026-10-04). `sdks/python/buf.gen.proto.yaml` pins `v0.9.0`. `bufbuild/plugins` has no directory newer than `v0.10.1`. Which of the three the BSR serves is **unverified**: the BSR plugin API answered an empty body to an unauthenticated query.
- **Go.** `sdks/go/buf.gen.go.yaml` pins `protocolbuffers/go:v1.36.9` and `connectrpc/go:v1.19.2`. `sdks/go/buf.gen.proto.yaml` and `pins.lock` pin `v1.36.11` and `v1.21.0`. There are two templates with two pins each.
- **C++.** `sdks/cpp/buf.gen.yaml` pins `protocolbuffers/cpp:v36.1` (D639's runtime). The latest is v36.2. A bump is optional.

## Unverified

- **Remote plugin versions** come from the `plugins/<owner>/<name>/` directories of [`bufbuild/plugins`][bufbuild-plugins] on GitHub. The dates are the commits that added each version, not BSR publish times, which could not be queried. The BSR may serve a version a few hours after the commit.
- **Maven Central trusted publishing.** Sonatype's portal-token page names user tokens as the only way to publish and mentions no OIDC. A web search on 2026-10-13 found no announcement of OIDC trusted publishing for Central. "No" is the best available answer, not a confirmed one.
- **Java's latest version.** Maven Central's `maven-metadata.xml` says 1.84.2 (last updated 2026-10-08). GitHub's newest release page is v1.84.1. Either is less than 14 days old on 2026-10-13.

## Sources

Fetched 2026-10-13:

- GitHub Releases API: [connect-es][gh-connect-es], [connect-py][gh-connect-py], [connect-go][gh-connect-go], [connect-rust][gh-connect-rust], [connect-swift][gh-connect-swift], [connect-kotlin][gh-connect-kotlin], [connect-dart][gh-connect-dart], [grpc-java][gh-grpc-java], [grpc-dotnet][gh-grpc-dotnet], [grpc-dart][gh-grpc-dart], [grpc][gh-grpc], [protobuf][gh-protobuf].
- Registry APIs: npm, PyPI, crates.io, Maven Central (`maven-metadata.xml`), NuGet registration, pub.dev, RubyGems, Packagist, PECL REST and CocoaPods trunk. Each is linked in its row.
- Trusted publishing: npm, PyPI, crates.io, NuGet, pub.dev, RubyGems and Sonatype. Each is linked in its row.

[npm-connect]: https://www.npmjs.com/package/@connectrpc/connect
[pypi-connect]: https://pypi.org/project/connectrpc/
[crates-connect]: https://crates.io/crates/connectrpc
[mvn-connect-kotlin]: https://repo1.maven.org/maven2/com/connectrpc/connect-kotlin/maven-metadata.xml
[mvn-grpc-java]: https://repo1.maven.org/maven2/io/grpc/grpc-api/maven-metadata.xml
[nuget-grpc]: https://www.nuget.org/packages/Grpc.Net.Client
[pub-connect]: https://pub.dev/packages/connectrpc
[pub-grpc]: https://pub.dev/packages/grpc
[gem-grpc]: https://rubygems.org/gems/grpc
[pecl-grpc]: https://pecl.php.net/package/grpc
[packagist-grpc]: https://packagist.org/packages/grpc/grpc
[pod-grpc]: https://cocoapods.org/pods/gRPC-ProtoRPC
[gh-connect-es]: https://github.com/connectrpc/connect-es/releases
[gh-connect-py]: https://github.com/connectrpc/connect-py/releases
[gh-connect-go]: https://github.com/connectrpc/connect-go/releases
[gh-connect-rust]: https://github.com/connectrpc/connect-rust/releases
[gh-connect-swift]: https://github.com/connectrpc/connect-swift/releases
[gh-connect-kotlin]: https://github.com/connectrpc/connect-kotlin/releases
[gh-connect-dart]: https://github.com/connectrpc/connect-dart/releases
[gh-grpc-java]: https://github.com/grpc/grpc-java/releases
[gh-grpc-dotnet]: https://github.com/grpc/grpc-dotnet/releases
[gh-grpc-dart]: https://github.com/grpc/grpc-dart/releases
[gh-grpc]: https://github.com/grpc/grpc/releases
[gh-protobuf]: https://github.com/protocolbuffers/protobuf/releases
[bufbuild-plugins]: https://github.com/bufbuild/plugins/tree/main/plugins
[tp-npm]: https://docs.npmjs.com/trusted-publishers
[tp-pypi]: https://docs.pypi.org/trusted-publishers/
[tp-crates]: https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/
[tp-nuget]: https://learn.microsoft.com/nuget/nuget-org/trusted-publishing
[tp-pub]: https://dart.dev/tools/pub/automated-publishing
[tp-rubygems]: https://guides.rubygems.org/trusted-publishing/
[tp-maven]: https://central.sonatype.org/publish/generate-portal-token/
