# .io Game

TODO

## Tech Stack

### Backend

- Backend written in Elixir
- Phoenix
  - Phoenix Channels for communication between the frontend + backend
  - Capn'Proto library for Erlang: http://ecapnp.astekk.se/

### Messaging + Serialization

- WebSocket-based communication between the backend and the frontend using Phoenix channels
- Binary message serialization using [Capn'Proto](https://capnproto.org/)
- All messages are tagged with a UUID that corresponds to some entity.
  - The "universe" can be an entity for general messages as well (UUID 0000-0000...?)

### Frontend

- Built with Rust and compiled into WebAssembly
- Using the SDL library to abstract rendering to canvas
