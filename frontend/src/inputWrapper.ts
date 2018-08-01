/**
 * Creates input handlers for the game and hooks them into the WebAssembly.
 */

import { getCanvas } from './renderMethods';
import { continueInit, handleWsMsg } from './index';

const canvas = getCanvas('canvas-2d');

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

const body = document.getElementsByTagName('body')[0];

export const initEventHandlers = (engine: typeof import('./game_engine')) => {
  body.onmousedown = evt => engine.handle_mouse_down(evt.x, evt.y);
  body.onmouseup = evt => engine.handle_mouse_up(evt.x, evt.y);
  body.onmousemove = evt => engine.handle_mouse_move(evt.x, evt.y);

  body.onkeydown = evt => engine.handle_key_down(evt.keyCode);
  body.onkeyup = evt => engine.handle_key_up(evt.keyCode);
};

export const send_message = (message: Uint8Array) => gameSocket.send(message);
