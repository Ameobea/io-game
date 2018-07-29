export const getCanvas = () => document.getElementsByTagName('canvas')[0];

const canvas = getCanvas();

const ctx = (() => {
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    const errMsg = 'Unable to create 2D rendering context; this application cannot run!';
    alert(errMsg);
    throw Error(errMsg);
  }
  return ctx;
})();

export const clearCanvas = () => ctx.clearRect(0, 0, canvas.width, canvas.height);

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
  ctx.strokeStyle = color;
  ctx.fillStyle = color;
  ctx.fillRect(x, y, width, height);
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
  ctx.beginPath();
  const color = `rgb(${r},${g},${b})`;
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.arc(x, y, radius, startAngle, endAngle, counterClockwise);
  ctx.stroke();
};

export const render_line = (width: number, x1: number, y1: number, x2: number, y2: number) => {
  ctx.beginPath();
  ctx.lineWidth = width;
  ctx.moveTo(x1, y1);
  ctx.lineTo(x2, y2);
  ctx.stroke();
};

export const fill_poly = (r: number, g: number, b: number, vertex_coords: number[]) => {
  const color = `rgb(${r},${g},${b})`;
  ctx.fillStyle = color;

  ctx.beginPath();
  for (let i = 0; i < vertex_coords.length; i += 2) {
    ctx.lineTo(vertex_coords[i], vertex_coords[i + 1]);
  }
  ctx.closePath();
  ctx.fill();
};
