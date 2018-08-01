export const getCanvas = (id: string): HTMLCanvasElement =>
  document.getElementById(id) as HTMLCanvasElement;

const canvas2D: HTMLCanvasElement = getCanvas('canvas-2d') as HTMLCanvasElement;

const { ctx2D, ctx2DBackground } = (() => {
  const ctx = canvas2D.getContext('2d');
  const ctx2DBackground = canvas2D.getContext('2d');
  if (!ctx || !ctx2DBackground) {
    const errMsg = 'Unable to create 2D rendering context; this application cannot run!';
    alert(errMsg);
    throw Error(errMsg);
  }
  return { ctx2D: ctx, ctx2DBackground };
})();

export const clearCanvas = () => ctx2D.clearRect(0, 0, canvas2D.width, canvas2D.height);

export const render_quad = (
  r: number,
  g: number,
  b: number,
  x: number,
  y: number,
  width: number,
  height: number
) => {
  const color = `rgb(${r},${g},${b})`;
  ctx2D.strokeStyle = color;
  ctx2D.fillStyle = color;
  ctx2D.fillRect(x, y, width, height);
};

export const render_arc = (
  r: number,
  g: number,
  b: number,
  x: number,
  y: number,
  width: number,
  radius: number,
  startAngle: number,
  endAngle: number,
  counterClockwise: boolean
) => {
  ctx2D.beginPath();
  const color = `rgb(${r},${g},${b})`;
  ctx2D.strokeStyle = color;
  ctx2D.lineWidth = width;
  ctx2D.arc(x, y, radius, startAngle, endAngle, counterClockwise);
  ctx2D.stroke();
};

export const render_line = (
  r: number,
  g: number,
  b: number,
  width: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number
) => {
  ctx2D.beginPath();
  const color = `rgb(${r},${g},${b})`;
  ctx2D.strokeStyle = color;
  ctx2D.lineWidth = width;
  ctx2D.moveTo(x1, y1);
  ctx2D.lineTo(x2, y2);
  ctx2D.stroke();
};

export const fill_poly = (r: number, g: number, b: number, vertex_coords: number[]) => {
  const color = `rgb(${r},${g},${b})`;
  ctx2D.fillStyle = color;

  ctx2D.beginPath();
  for (let i = 0; i < vertex_coords.length; i += 2) {
    ctx2D.lineTo(vertex_coords[i], vertex_coords[i + 1]);
  }
  ctx2D.closePath();
  ctx2D.fill();
};

export const render_point = (r: number, g: number, b: number, x: number, y: number) => {
  const color = `rgb(${r},${g},${b})`;
  ctx2D.fillStyle = color;
  ctx2D.fillRect(x, y, 1, 1);
};
