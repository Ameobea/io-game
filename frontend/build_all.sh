cd game-engine \
  && ./release.sh \
  && wasm-bindgen ./target/wasm32-unknown-unknown/release/game_engine.wasm --out-dir ./build
cd -
cp ./game-engine/build/* ./src
wasm-opt ./src/*.wasm -O4 -o ./src/*.wasm
yarn build || npm build
