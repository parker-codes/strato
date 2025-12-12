# Strato Client

---

## Dev

This project uses [Leptos](https://leptos.dev/) (a Rust-Wasm framework).

Run `trunk serve` which bundles the assets and serves at [localhost:8080](http://localhost:8080).

## Release

First build the CSS with `pnpm run tw:build`. Then build the app with `trunk build --release`. Will build locally and push web assets to hosting provider for simplicity of build tooling.
