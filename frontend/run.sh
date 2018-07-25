cd game-engine \
  && ./build.sh \
  && wasm-gc target/wasm32-unknown-unknown/debug/game_engine.wasm \
  && wasm-bindgen ./target/wasm32-unknown-unknown/debug/game_engine.wasm --out-dir ./build
cd -
cp ./game-engine/build/* ./src/
yarn start
