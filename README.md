# Food DSL

## Developing

### Set up dependencies

#### With Nix

Run `nix develop` from the repository root, and run the commands below inside that shell.

#### Without Nix

1. Set up Rust via [rustup](https://rustup.rs):
```sh
rustup target add wasm32-unknown-unknown
```
2. Install [wasm-pack](https://github.com/wasm-bindgen/wasm-pack):
```sh
cargo install wasm-pack
```
3. Follow the instructions at [Node.js](https://nodejs.org) (version 24) and set up a package manager, e.g.,
```sh
npm install -g pnpm
```
4. Follow the instructions at [Elm](https://elm-lang.org/) to install Elm.

### Build the Rust core

At the repository root:
```sh
cd binding
wasm-pack build --target web --release
cd ..
```

Run this again whenever code in `dsl/` or `binding/` changes.

### Build the UI

```sh
cd ui
pnpm install
pnpm dev
```

Then go to <http://localhost:5173>! 🍰

#### Some commands...

From `ui/`, you can run:

| Command | Description |
| --- | --- |
| `pnpm check` | Lints and checks formatting |
| `pnpm typecheck` | Type-checks the TypeScript |
| `pnpm build` | Builds the optimized site into `ui/dist` |

![Farm to table software](https://jlubin.net/assets/farm-to-table.png)
