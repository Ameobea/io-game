cd game-engine \
  && ./build.sh \
  && wasm-bindgen ./target/wasm32-unknown-unknown/debug/game_engine.wasm --out-dir ./build
cd -
cp ./game-engine/build/* ./src/
yarn start
