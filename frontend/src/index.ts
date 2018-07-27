const wasm = import('./game_engine');
import { getCanvas, clearCanvas } from './renderMethods';
import { initEventHandlers } from './inputWrapper';

const canvas = getCanvas();

export const timer = timeMs => new Promise(f => setTimeout(f, timeMs));

export let continueInit: () => void;

const wsInitPromise = new Promise(f => {
  continueInit = f;
});

export let handleWsMsg: (msg: ArrayBuffer) => void;

wasm
  .then(async engine => {
    (window as any).handle_message = engine.handle_channel_message;

    // Wait for the websocket to connect
    await wsInitPromise;

    // Initialize internal game state and provide better error messages when the underlying Rust
    // code panics.
    engine.init();
    initEventHandlers(engine);
    handleWsMsg = (ab: ArrayBuffer) => engine.handle_channel_message(new Uint8Array(ab));

    const tick = () => {
      clearCanvas();
      engine.tick();
      requestAnimationFrame(tick);
    };

    tick();
  })
  .catch(err => console.error(`Error while loading Wasm module: ${err}`));
