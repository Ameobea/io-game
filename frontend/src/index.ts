const wasm = import('./game_engine');
import { clearCanvas } from './renderMethods';
import { initWebGL } from './webgl';

export const timer = timeMs => new Promise(f => setTimeout(f, timeMs));

export let continueInit: () => void;

const wsInitPromise = new Promise(f => {
  continueInit = f;
});

export let handleWsMsg: (msg: ArrayBuffer) => void;

let engineHandle: typeof import('./game_engine');

export const getEngine = (): typeof import('./game_engine') => engineHandle;

let tick;

export const start_game_loop = () => tick();

wasm
  .then(async engine => {
    engineHandle = engine;

    const { canvasHeight, canvasWidth } = initWebGL();

    // Wait for the websocket to connect
    await wsInitPromise;

    // Initialize internal game state and provide better error messages when the underlying Rust
    // code panics.
    engine.init(canvasHeight, canvasWidth);
    handleWsMsg = (ab: ArrayBuffer) => engine.handle_channel_message(new Uint8Array(ab));

    tick = () => {
      clearCanvas();
      engine.tick();
      requestAnimationFrame(tick);
    };
  })
  .catch(err => console.error(`Error while loading Wasm module: ${err}`));
