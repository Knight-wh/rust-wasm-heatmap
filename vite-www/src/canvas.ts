import { HeatMap, HeatPoint, RGBA } from "rust-wasm-heatmap";
import { memory } from "rust-wasm-heatmap/rust_wasm_heatmap_bg.wasm";

const canvasHeight = 400;
const canvasWidth = 400;

const minX = 0.0,
  minY = 0.0,
  maxX = 10.0,
  maxY = 10.0,
  maxHeat = 20.0;
const heatmap = HeatMap.new(200, 10.0, minX, minY, maxX, maxY, maxHeat);
const gradient = ["00AAFF", "00FF00", "FFFF00", "FF8800", "FF0000"];
const rgbas = parseGradient(gradient).map((g) => RGBA.new(g.r, g.g, g.b, 255));
heatmap.set_gradients(rgbas);

export function setupCanvas(canvas: HTMLCanvasElement) {
  canvas.width = canvasWidth;
  canvas.height = canvasHeight;

  const hw = heatmap.width();
  const hh = heatmap.height();
  console.log("width", hw);
  console.log("height", hh);

  const points = [];
  for (let i = 0; i < 50; i++) {
    let x = Math.random() * (maxX - minX) + minX;
    let y = Math.random() * (maxY - minY) + minY;
    points.push(HeatPoint.new(x, y, Math.random() * maxHeat));
  }
  heatmap.add_points(points);
  const ctx = canvas.getContext("2d");
  // Draw
  if (!ctx) return;
  const colorsPtr = heatmap.color_values();
  const colorsArr = new Uint8ClampedArray(
    memory.buffer,
    colorsPtr,
    4 * hw * hh
  );
  ctx.strokeRect(0, 0, hw, hh);
  const imageData = new ImageData(colorsArr, hw, hh);
  createImageBitmap(imageData, { imageOrientation: "flipY" }).then((bitmap) => {
    ctx.drawImage(bitmap, 0, 0);
  });
}

function parseGradient(
  gradient: string[] | number[][]
): { r: number; g: number; b: number }[] {
  return gradient.map((color) => {
    if (color.toString().match(/^#?[0-9a-f]{3}$/i)) {
      color = color.toString().replace(/^#?(.)(.)(.)$/, "$1$1$2$2$3$3");
    }
    if (typeof color === "string") {
      if (color.match(/^#?[0-9a-f]{6}$/i)) {
        // @ts-ignore
        color = color
          .match(/^#?(..)(..)(..)$/)
          .slice(1)
          .map((n) => parseInt(n, 16));
      } else {
        throw Error(`Invalid color format (${color}).`);
      }
    } else if (color instanceof Array) {
      if (
        !(
          color.length &&
          isUint8(color[0]) &&
          isUint8(color[1]) &&
          isUint8(color[2])
        )
      ) {
        throw Error(`Invalid color format (${JSON.stringify(color)}).`);
      }
    } else {
      throw Error(`Invalid color object (${JSON.stringify(color)}).`);
    }
    return { r: color[0], g: color[1], b: color[2] };
  });
}

function isUint8(num: number) {
  return typeof num == "number" && 0 <= num && num >= 255;
}
