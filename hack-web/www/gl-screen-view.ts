import { HackEmulator } from "../pkg";

const vsSource = `
attribute vec4 a_position;
attribute vec2 a_texcoord;

varying vec2 v_texcoord;

void main() {
  gl_Position = a_position;
  v_texcoord = a_texcoord;
}
`;

const fsSource = `
precision mediump float;

varying vec2 v_texcoord;
uniform vec2 u_resolution;
uniform vec2 u_texsize;
uniform sampler2D u_texture;

void main() {
  // Figure out the corresponding x-axis pixel in the screen
  float texel_coord = floor(v_texcoord.x * u_resolution.x);

  // Texels are 16-bit Hack computer words, so calculate the bit offset
  float bit_index = mod(texel_coord, 16.0);

  // Retrieve the 16-bit word that contains this pixel's value.
  vec4 texel = texture2D(u_texture, v_texcoord);

  // There's no native support for 16-bit uints, so instead it's encoded
  // as four 4-bit RGBA values.
  float nibble = texel.r;
  if (bit_index < 12.0) {
    nibble = texel.g;
  }
  if (bit_index < 8.0) {
    nibble = texel.b;
  }
  if (bit_index < 4.0) {
    nibble = texel.a;
  }

  // The RGBA values are normalized to [0.0, 1.0], so map them back to the
  // original domain, then extract the relevant bit to get the pixel value.
  float pixel = mod(floor((nibble * 255.0) / pow(2.0, mod(bit_index , 4.0))), 2.0);

  gl_FragColor = vec4(0.0, pixel, 0.0, 1.0);
}
`;

export class GLScreenView {
  el: HTMLCanvasElement;
  gl: WebGLRenderingContext;
  shaderProgram: WebGLProgram;
  texture: WebGLTexture;
  emulator: HackEmulator;

  constructor(emulator: HackEmulator) {
    this.emulator = emulator;
    this.el = document.createElement('canvas');
    this.el.width = 1024;
    this.el.height = 512;
    this.el.style.width = '512px';
    this.el.style.height = '256px';
    const gl = this.el.getContext('webgl');
    if (!gl) {
      throw new Error("Couldn't get webgl context");
    }
    this.gl = gl;

    this.gl.clearColor(0.0, 0.0, 0.0, 1.0);

    this.shaderProgram = this.initShaders();
    this.initVertexBuffers();
    this.texture = this.initTexture();

    this.update();
  }

  update() {
    this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    this.loadTexture();
    this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
  }

  initShaders() {
    const vertexShader = this.compileShader(this.gl.VERTEX_SHADER, vsSource);
    const fragmentShader = this.compileShader(this.gl.FRAGMENT_SHADER, fsSource);
    const shaderProgram = this.linkShaderProgram(vertexShader, fragmentShader);
    this.gl.useProgram(shaderProgram);

    return shaderProgram;
  }

  compileShader(type: number, source: string) {
    const shader = this.gl.createShader(type)!;

    this.gl.shaderSource(shader, source);
    this.gl.compileShader(shader);

    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
      const msg = 'An error occurred compiling the shaders: ' + this.gl.getShaderInfoLog(shader);
      this.gl.deleteShader(shader);
      throw new Error(msg);
    }

    return shader;
  }

  linkShaderProgram(vertexShader: WebGLShader, fragmentShader: WebGLShader) {
    const shaderProgram = this.gl.createProgram()!;
    this.gl.attachShader(shaderProgram, vertexShader);
    this.gl.attachShader(shaderProgram, fragmentShader);
    this.gl.linkProgram(shaderProgram);

    if (!this.gl.getProgramParameter(shaderProgram, this.gl.LINK_STATUS)) {
      this.gl.deleteProgram(shaderProgram);
      throw new Error('Unable to initialize the shader program: ' + this.gl.getProgramInfoLog(shaderProgram));
    }

    return shaderProgram;
  }

  initVertexBuffers() {
    // Set up vertex positions for the rectangle that we render the screen to
    const positionBuffer = this.gl.createBuffer();
    this.gl.bindBuffer(this.gl.ARRAY_BUFFER, positionBuffer);

    const positions = [
      -1.0, -1.0, // bottom left
      1.0, -1.0,  // bottom right
      -1.0, 1.0,  // top left
      1.0, 1.0,   // top right
    ];
    this.gl.bufferData(this.gl.ARRAY_BUFFER, new Float32Array(positions), this.gl.STATIC_DRAW);

    const positionAttributeLocation = this.gl.getAttribLocation(this.shaderProgram, "a_position");
    this.gl.enableVertexAttribArray(positionAttributeLocation);
    this.gl.vertexAttribPointer(positionAttributeLocation, 2, this.gl.FLOAT, false, 0, 0);
  }

  initTexture() {
    const texture = this.gl.createTexture()!;

    // Set up the texture mapping coordinates, which don't change over time
    const texCoordBuffer = this.gl.createBuffer();
    this.gl.bindBuffer(this.gl.ARRAY_BUFFER, texCoordBuffer);
    this.gl.bufferData(this.gl.ARRAY_BUFFER, new Float32Array([
      // texture coordinates for vertices (flipped in the y axis)
      0, 1, // bottom left
      1, 1, // bottom right
      0, 0, // top left
      1, 0, // top right
    ]), this.gl.STATIC_DRAW);
    const texCoordLoc = this.gl.getAttribLocation(this.shaderProgram, 'a_texcoord');
    this.gl.vertexAttribPointer(texCoordLoc, 2, this.gl.FLOAT, false, 0, 0);
    this.gl.enableVertexAttribArray(texCoordLoc);

    // `texsize` is the size of the texture in 16-bit words
    const uTexsizeLoc = this.gl.getUniformLocation(this.shaderProgram, 'u_texsize');
    this.gl.uniform2f(uTexsizeLoc, 32, 256);

    // `resolution` is the actual pixel resolution of the screen. As one word
    // holds 16 pixel values, the x-axis expands from 32 to 32 * 16 = 512.
    const uResolutionLoc = this.gl.getUniformLocation(this.shaderProgram, 'u_resolution');
    this.gl.uniform2f(uResolutionLoc, 512, 256);

    return texture;
  }

  loadTexture() {
    // Pass the screen memory (vram) straight to the GPU as a texture, then
    // unpack the 16-bit words into pixel values in the fragement shader.
    const data = this.emulator.memory.slice(0x4000, 0x6000);
    this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);

    // WebGL 1 isn't very flexible about texture formats, so we use one of the
    // (few) 16-bit formats available. In the fragment shader, we'll be able to
    // extract each 16-bit word using the texture sampler, however the 16 bits
    // will be split into r, b, g, and a values of 4 bits each.
    this.gl.texImage2D(this.gl.TEXTURE_2D, 0, this.gl.RGBA, 32, 256, 0, this.gl.RGBA, this.gl.UNSIGNED_SHORT_4_4_4_4, data);

    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);

    // It's critical we use NEAREST rather than LINEAR as the texture is data
    // rather than an image, so interpolation is a big no-no.
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST);
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.NEAREST);
  }
}