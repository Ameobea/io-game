# Frontend

This contains the full frontend of the application. It consists of a simple HTML page that loads the Phoenix JS client library, the wrapper JS used to connect to the backend, and the WebAssembly blob that contains the main game code.

## Building

Building the frontend consists of a few steps. First, the Rust code that defines the game engine needs to be compiled. Next, the `wasm-bindgen` command line utility must be used to generate JavaScript wrappers TypeScript definitions for the exported Rust functions. Finally, the entire thing is bundled with webpack and put into the `./dist` directory.

You can build the whole thing by running `./build_all.sh`
