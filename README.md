# Snake - Rust WebAssembly game

Snake is being built as a browser game with a reusable Rust game engine and a
WebAssembly frontend.

Build the WebAssembly target:

```bash
cargo build --release --target wasm32-unknown-unknown
```

## Release build

Install [Trunk](https://trunkrs.dev/) and create a self-contained static build:

```bash
cargo install trunk --locked
./scripts/build-web.sh
```

The release script requires a clean worktree and writes optimized assets plus
`game-manifest.json` to `dist/`. The manifest records the exact source commit,
repository URL, and UTC build time.

To copy a release into a static resume website, see
[`docs/resume-integration.md`](docs/resume-integration.md).

## Verification

```bash
cargo fmt --check
cargo test
cargo clippy --target wasm32-unknown-unknown -- -D warnings
trunk build --release
```
