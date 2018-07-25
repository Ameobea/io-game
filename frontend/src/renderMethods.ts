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
