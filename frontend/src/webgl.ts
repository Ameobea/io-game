/**
 * Much of this code was taken or adapted from https://webglfundamentals.org/ and is used under
 * the terms of the license attached below:
 *
 * Copyright 2012, Gregg Tavares.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Gregg Tavares. nor the names of his
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

const vertexShaderSrc = require('./shaders/vert.glsl');
const fragShaderSrc = require('./shaders/frag.glsl');

import { getCanvas } from './renderMethods';

const canvasWebGL: HTMLCanvasElement = getCanvas('canvas-webgl') as HTMLCanvasElement;

const gl = (() => {
  const ctx = canvasWebGL.getContext('webgl');
  if (!ctx) {
    const errMsg = 'Unable to create WebGL rendering context; this application cannot run!';
    alert(errMsg);
    throw Error(errMsg);
  }
  return ctx;
})();

/**
 * Compiles either a shader of type `gl.VERTEX_SHADER` or `gl.FRAGMENT_SHADER`.
 */
const createShader = (sourceCode: string, type: number): WebGLShader => {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, sourceCode);
  gl.compileShader(shader);

  if (!shader || !gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = shader && gl.getShaderInfoLog(shader);
    throw 'Could not compile WebGL program. \n\n' + info;
  }
  return shader;
};

const localState: { [key: string]: any } = {};

export const create_background_texture = async (
  height: number,
  width: number,
  textureData: Uint8Array
) => {
  // Create an initialize a WebGL texture
  const texture = gl.createTexture();
  if (!texture) {
    throw 'Unable to create WebGL texture';
  }
  localState.backgroundTexture = texture;
  localState.backgroundHeight = height;
  localState.backgroundWidth = width;
  gl.bindTexture(gl.TEXTURE_2D, texture);

  // Populate the texture with the pixel data generated in Wasm
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    width,
    height,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    textureData
  );

  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
};

// Taken from https://webglfundamentals.org/webgl/resources/m4.js
function orthographic(left, right, bottom, top, near, far, dst?: Float32Array) {
  dst = dst || new Float32Array(16);

  dst[0] = 2 / (right - left);
  dst[1] = 0;
  dst[2] = 0;
  dst[3] = 0;
  dst[4] = 0;
  dst[5] = 2 / (top - bottom);
  dst[6] = 0;
  dst[7] = 0;
  dst[8] = 0;
  dst[9] = 0;
  dst[10] = 2 / (near - far);
  dst[11] = 0;
  dst[12] = (left + right) / (left - right);
  dst[13] = (bottom + top) / (bottom - top);
  dst[14] = (near + far) / (near - far);
  dst[15] = 1;

  return dst;
}

// Taken from https://webglfundamentals.org/webgl/resources/m4.js
function scale(m, sx, sy, sz, dst?: Float32Array) {
  // This is the optimized verison of
  // return multiply(m, scaling(sx, sy, sz), dst);
  dst = dst || new Float32Array(16);

  dst[0] = sx * m[0 * 4 + 0];
  dst[1] = sx * m[0 * 4 + 1];
  dst[2] = sx * m[0 * 4 + 2];
  dst[3] = sx * m[0 * 4 + 3];
  dst[4] = sy * m[1 * 4 + 0];
  dst[5] = sy * m[1 * 4 + 1];
  dst[6] = sy * m[1 * 4 + 2];
  dst[7] = sy * m[1 * 4 + 3];
  dst[8] = sz * m[2 * 4 + 0];
  dst[9] = sz * m[2 * 4 + 1];
  dst[10] = sz * m[2 * 4 + 2];
  dst[11] = sz * m[2 * 4 + 3];

  if (m !== dst) {
    dst[12] = m[12];
    dst[13] = m[13];
    dst[14] = m[14];
    dst[15] = m[15];
  }

  return dst;
}

// Taken from https://webglfundamentals.org/webgl/resources/m4.js
function translation(tx, ty, tz, dst?: Float32Array) {
  dst = dst || new Float32Array(16);

  dst[0] = 1;
  dst[1] = 0;
  dst[2] = 0;
  dst[3] = 0;
  dst[4] = 0;
  dst[5] = 1;
  dst[6] = 0;
  dst[7] = 0;
  dst[8] = 0;
  dst[9] = 0;
  dst[10] = 1;
  dst[11] = 0;
  dst[12] = tx;
  dst[13] = ty;
  dst[14] = tz;
  dst[15] = 1;

  return dst;
}

export const initWebGL = () => {
  localState.backgroundProgram = gl.createProgram();
  const backgroundProgram = localState.backgroundProgram;

  gl.attachShader(backgroundProgram, createShader(vertexShaderSrc, gl.VERTEX_SHADER));
  gl.attachShader(backgroundProgram, createShader(fragShaderSrc, gl.FRAGMENT_SHADER));

  gl.linkProgram(backgroundProgram);

  // look up where the vertex data needs to go.
  localState.positionLocation = gl.getAttribLocation(backgroundProgram, 'a_position');
  localState.texcoordLocation = gl.getAttribLocation(backgroundProgram, 'a_texcoord');

  // lookup uniforms
  localState.matrixLocation = gl.getUniformLocation(backgroundProgram, 'u_matrix');
  localState.textureMatrixLocation = gl.getUniformLocation(backgroundProgram, 'u_textureMatrix');
  localState.textureLocation = gl.getUniformLocation(backgroundProgram, 'u_texture');

  // Create a buffer.
  localState.positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, localState.positionBuffer);

  // Put a unit quad in the buffer
  localState.positions = [0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1];
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(localState.positions), gl.STATIC_DRAW);

  // Create a buffer for texture coords
  localState.texcoordBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, localState.texcoordBuffer);

  // Put texcoords in the buffer
  localState.texcoords = [0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1];
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(localState.texcoords), gl.STATIC_DRAW);

  if (!backgroundProgram) {
    throw 'Unable to create WebGL program';
  }

  if (!gl.getProgramParameter(backgroundProgram, gl.LINK_STATUS)) {
    throw `Could not compile WebGL program. \n\n'${gl.getProgramInfoLog(backgroundProgram)}`;
  }

  gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
};

export const draw_background = (offsetX: number, offsetY: number) => {
  gl.bindTexture(gl.TEXTURE_2D, localState.backgroundTexture);

  // Tell WebGL to use our shader program pair
  gl.useProgram(localState.backgroundProgram);

  gl.bindBuffer(gl.ARRAY_BUFFER, localState.positionBuffer);
  gl.enableVertexAttribArray(localState.positionLocation);
  gl.vertexAttribPointer(localState.positionLocation, 2, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, localState.texcoordBuffer);
  gl.enableVertexAttribArray(localState.texcoordLocation);
  gl.vertexAttribPointer(localState.texcoordLocation, 2, gl.FLOAT, false, 0, 0);

  // this matirx will convert from pixels to clip space
  let matrix = orthographic(0, gl.canvas.width, gl.canvas.height, 0, -1, 1);

  // this matrix will scale our 1 unit quad from 1 unit to texWidth, texHeight units
  matrix = scale(matrix, gl.canvas.width, gl.canvas.height, 1);

  // Set the matrix.
  gl.uniformMatrix4fv(localState.matrixLocation, false, matrix);

  // Because texture coordinates go from 0 to 1 and because our texture coordinates are already a
  // unit quad, we can select an area of the texture by scaling the unit quad down.
  var texMatrix = translation(
    (offsetX * 0.333333) / localState.backgroundWidth,
    (offsetY * 0.333333) / localState.backgroundHeight,
    0
  );
  texMatrix = scale(
    texMatrix,
    gl.canvas.width / localState.backgroundWidth,
    gl.canvas.height / localState.backgroundHeight,
    1
  );

  // Set the texture matrix.
  gl.uniformMatrix4fv(localState.textureMatrixLocation, false, texMatrix);

  // Tell the shader to get the texture from texture unit 0
  gl.uniform1i(localState.textureLocation, 0);

  // draw the quad (2 triangles, 6 vertices)
  gl.drawArrays(gl.TRIANGLES, 0, 6);
};
