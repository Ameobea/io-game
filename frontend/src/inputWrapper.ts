/**
 * Creates input handlers for the game and hooks them into the WebAssembly.
 */

import { getCanvas } from './renderMethods';
import { continueInit, handleWsMsg } from './index';

const canvas = getCanvas();

const gameSocket = new WebSocket('ws://localhost:4000/socket/websocket?vsn=1.0.0');

gameSocket.binaryType = 'arraybuffer';

gameSocket.onmessage = evt => {
  if (!(evt.data instanceof ArrayBuffer)) {
    console.error(`Received non-binary message from websocket: ${evt.data}`);
    return;
  }

  const data = evt.data as ArrayBuffer;
  handleWsMsg(data);
};

gameSocket.onerror = evt => console.error('WebSocket error:', evt);

gameSocket.onopen = () => continueInit();

export const initEventHandlers = (engine: typeof import('./game_engine')) => {
  canvas.addEventListener('mousedown', evt => engine.handle_mouse_down(evt.x, evt.y));
  canvas.addEventListener('mouseup', evt => engine.handle_mouse_up(evt.x, evt.y));
  canvas.addEventListener('mousemove', evt => engine.handle_mouse_move(evt.x, evt.y));

  document.addEventListener('keydown', evt => engine.handle_key_down(evt.keyCode));
  document.addEventListener('keyup', evt => engine.handle_key_up(evt.keyCode));
};

export const send_message = (message: Uint8Array) =>
  console.log(message) || gameSocket.send(message);
