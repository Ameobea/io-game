# .io Game

TODO

## Tech Stack

### Backend

- Backend written in Elixir
- Phoenix
  - Phoenix Channels for communication between the frontend + backend

### Messaging + Serialization

- WebSocket-based communication between the backend and the frontend using Phoenix channels
- Binary message serialization using Protcol Buffers.
- All messages are tagged with a UUID that corresponds to some entity.
  - The "universe" can be an entity for general messages as well (UUID 0000-0000...?)

### Frontend

- Built with Rust and compiled into WebAssembly
- Uses the Phoenix channels JS client library as a wrapper to connect to the backend and relay data to the WebAssembly code that powers the game.

## Installation

### Backend

- Install Elixir with `brew install elixir`
- Install all deps within `backend` with `mix deps.get`
- Compile and start server with `mix phx.server`

### Frontend

- You'll need to install Rust in order to build the frontend. I suggest using [Rustup](https://rustup.rs/).
- Run `rustup toolchain install nightly` to install the nightly toolchain
- Run `rustup default nightly` to make the nightly toolchain default.
- Run `rustup target add wasm32-unknown-unknown --toolchain nightly` to add the WebAssembly target.
- Run `cargo install wasm-bindgen-cli` to install the `wasm-bindgen-cli` which is used to generate JS wrapper code and TypeScript definitions from WebAssembly files.
- Run `cargo install wasm-gc` to install the `wasm-gc` utility which is used to strip unused functions from wasm files.
