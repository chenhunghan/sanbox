# sanbox

> A sandbox by agents, for agents, to play in sand.

<img width="600" src="https://github.com/user-attachments/assets/3efae76c-1687-4365-8ea0-b3cebfb1a07f" />

`sanbox` is a project that renders a "sandbox"

## Demo

- Web: https://chenhunghan.github.io/sanbox/
- Releases: https://github.com/chenhunghan/sanbox/releases

## CLI

The binary name is `san`.

Run it with a profile word and an optional extra word:

```bash
san claude box
san codex box
san openclaw box
```

You can also run it directly from source:

```bash
cargo run --release -- claude box
```

Controls:

- `q` to quit
- `Esc` to quit
- `Ctrl+C` to quit

## Install

### Download a prebuilt binary

Grab the latest archive from the releases page:

- Linux: `san-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- macOS Apple Silicon: `san-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- macOS Intel: `san-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- Windows: `san-vX.Y.Z-x86_64-pc-windows-msvc.zip`

### Build locally

Requirements:

- Rust toolchain
- Cargo

Then:

```bash
cargo build --release
./target/release/san claude box
```

## Web Demo

The site in `docs/` uses the same Rust simulation compiled to WebAssembly.

Build the web bundle:

```bash
bash ./scripts/build-web.sh
```

Preview it locally:

```bash
python3 -m http.server 4173 -d docs
```

Then open:

```text
http://127.0.0.1:4173/
```

## Project Layout

```text
src/main.rs                  terminal CLI entrypoint
src/sim.rs                   shared sand simulation
src/renderer.rs              terminal renderer
src/web.rs                   wasm bindings for the browser
docs/                        static site published to GitHub Pages
scripts/build-web.sh         local wasm build script
.github/workflows/           Pages deploy + release automation
```

## Release Flow

This repo uses `release-please` for versioning and GitHub Releases.

- commits merged to `main` should use conventional commit prefixes such as `feat:` and `fix:`
- `release-please` opens a release PR
- merging that PR creates a GitHub release
- the workflow then builds CLI archives for Linux, macOS, and Windows and uploads them to the release page

## GitHub Pages

The website is deployed from GitHub Actions to:

https://chenhunghan.github.io/sanbox/

The workflow builds the wasm bundle into `docs/` and publishes that directory to GitHub Pages.
