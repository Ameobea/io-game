/**
 * Creates input handlers for the game and hooks them into the WebAssembly.
 */

import { getCanvas } from './renderMethods';
import { gameSocket } from '.';

const canvas = getCanvas();

export const initEventHandlers = (engine: typeof import('./game_engine')) => {
  canvas.addEventListener('mousedown', evt => engine.handle_mouse_down(evt.x, evt.y));
  canvas.addEventListener('mouseup', evt => engine.handle_mouse_up(evt.x, evt.y));
  canvas.addEventListener('mousemove', evt => engine.handle_mouse_move(evt.x, evt.y));

  document.addEventListener('keydown', evt => engine.handle_key_down(evt.keyCode));
  document.addEventListener('keyup', evt => engine.handle_key_up(evt.keyCode));
};

export const send_message = (message: Uint8Array) => gameSocket.push(message);