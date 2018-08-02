const wasm = import('./game_engine');
import { clearCanvas } from './renderMethods';
import { initWebGL } from './webgl';
import { initEventHandlers } from './inputWrapper';

export const timer = timeMs => new Promise(f => setTimeout(f, timeMs));

export let continueInit: () => void;

const wsInitPromise = new Promise(f => {
  continueInit = f;
});

export let handleWsMsg: (msg: ArrayBuffer) => void;

const createAsteroidSpawner = engine => (x: number, y: number) =>
  engine.spawn_asteroid(
    new Float32Array(
      [0, -20, 5, -14, 10, -7, 15, 0, 8, 8, 3, 17, 0, 18, -5, 12, -8, -2, -5, -12].map(
        i => i * 2.2 + 10
      )
    ),
    x,
    y,
    0.0,
    0.0,
    0.0,
    Math.abs(Math.random() * 0.02)
  );

wasm
  .then(async engine => {
    (window as any).handle_message = engine.handle_channel_message;

    const { canvasHeight, canvasWidth } = initWebGL();

    // Wait for the websocket to connect
    await wsInitPromise;

    // Initialize internal game state and provide better error messages when the underlying Rust
    // code panics.
    engine.init(canvasHeight, canvasWidth);
    initEventHandlers(engine);
    handleWsMsg = (ab: ArrayBuffer) => engine.handle_channel_message(new Uint8Array(ab));

    const tick = () => {
      clearCanvas();
      engine.tick();
      requestAnimationFrame(tick);
    };

    tick();

    const spawnAsteroid = createAsteroidSpawner(engine);
    spawnAsteroid(300, 400);
    spawnAsteroid(100, 200);
    spawnAsteroid(240, 120);
    spawnAsteroid(60, 360);
    spawnAsteroid(140, 300);
    spawnAsteroid(300, 80);
  })
  .catch(err => console.error(`Error while loading Wasm module: ${err}`));
