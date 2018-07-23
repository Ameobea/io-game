import { Socket } from 'phoenix-socket';

const wasm = import('./game_engine');
import { clearCanvas } from './renderMethods';

const timer = timeMs => new Promise(f => setTimeout(f, timeMs));

wasm.then(async engine => {
  const tick = () => {
    clearCanvas();
    engine.tick();
    requestAnimationFrame(tick);
  };

  tick();

  await timer(500);
  const msg1 = engine.temp_gen_server_message_1();
  console.log(msg1);
  engine.handle_message(msg1);

  await timer(750);
  const msg2 = engine.temp_gen_server_message_2();
  console.log(msg2);
  console.log(engine.handle_message);
  engine.handle_message(msg2);

  ////////

  console.log('Initializing WS connection to game server...');
  const socket = new Socket('ws://localhost:4000/socket');
  socket.onError = console.error;
  socket.onConnError = console.error;
  socket.connect();

  const game = socket.channel('game:first');
  const join = game.join();
  console.log(join);
  join
    .receive('ok', () => console.log('Connected to lobby!'))
    .receive('error', (reasons: any) => console.error('create failed', reasons));

  (window as any).alex = () => {
    game.push('move_up');
  };
  game.on('temp_gen_server_message_1_res', res => {
    console.log(res);
    engine.handle_message(res);
  });
  (window as any).alex2 = () => {
    game.push('temp_gen_server_message_1');
  };
});
