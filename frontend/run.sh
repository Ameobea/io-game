cd game-engine \
  && ./release.sh \
  && wasm-bindgen ./target/wasm32-unknown-unknown/release/game_engine.wasm --out-dir ./build
cd - && cp ./index.html ./dist/index.html \
  && yarn start
