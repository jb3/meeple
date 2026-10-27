# Meeple

An ambient, procedurally generated isometric voxel world for the background of
your screen. Villages, towns, farms, railways and the people who live there go
about their day while the camera drifts between them. Written in Rust, compiled
to WebAssembly and rendered with WebGL2; the soundscape is synthesised live
with WebAudio.

**[Try it in your browser: meeple.jb3.dev](https://meeple.jb3.dev)**

> [!WARNING]
> This entire project is vibe-coded. No guarantees are made about its
> efficiency, correctness or security, or that it is free of bugs. Use it at
> your own risk.

## Running

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked   # or: brew install trunk
trunk serve --release
```

Open <http://localhost:8080> and click once to enable sound. Press `H` or `?`
in the app for the controls.

`make test` runs the native tests; `make` builds a release bundle into `release/`.

## Documentation

Everything else is in [`docs/`](docs/README.md): the feature list, controls and
URL options, and how each part of the program works.

## Licence

[MIT](LICENSE)
