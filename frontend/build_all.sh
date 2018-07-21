cd game-engine \
  && ./release.sh \
  && wasm-bindgen ./target/wasm32-unknown-unknown/release/game_engine.wasm --out-dir ./build
yarn build || npm build
cd - && cp ./index.html ./dist/index.html
