import { HeatMapV2, RGBA } from "rust-wasm-heatmap";
import { memory } from "rust-wasm-heatmap/rust_wasm_heatmap_bg.wasm";
import { parseGradient } from "./utils";

const canvasHeight = 400;
const canvasWidth = 400;

const minX = 0.0,
  minY = 0.0,
  maxX = 10.0,
  maxY = 10.0,
  maxHeat = 20.0,
  radius = 20.0,
  size = 400;
const hmap = HeatMapV2.new();
const gradient = ["00AAFF", "00FF00", "FFFF00", "FF8800", "FF0000"];
const rgbas = parseGradient(gradient).map((g) => RGBA.new(g.r, g.g, g.b, 255));
hmap.set_gradients(rgbas);
hmap.set_radius(radius);
hmap.set_max_heat(maxHeat);
hmap.set_flip_y(false);
hmap.set_size(size, minX, minY, maxX, maxY);

export async function setupCanvas() {
  const canvas = document.querySelector<HTMLCanvasElement>("#myCanvas")!;
  canvas.width = canvasWidth;
  canvas.height = canvasHeight;

  const hw = hmap.width();
  const hh = hmap.height();
  console.log("width", hw);
  console.log("height", hh);

  const points = [];
  for (let i = 0; i < 50; i++) {
    let x = Math.random() * (maxX - minX) + minX;
    let y = Math.random() * (maxY - minY) + minY;
    let heat = Math.random() * maxHeat;
    points.push(x, y, heat);
  }
  hmap.add_points(new Float64Array(points));
  hmap.calc_heatmap();
  const colorsPtr = hmap.color_values();
  const uintc8 = new Uint8ClampedArray(memory.buffer, colorsPtr, 4 * hw * hh);

  const ctx = canvas.getContext("2d");
  // Draw
  if (!ctx) return;
  ctx.strokeRect(0, 0, hw, hh);
  // Draw Heat map
  const imageData = new ImageData(uintc8, hw, hh);
  const bitmap = await createImageBitmap(imageData);
  ctx.drawImage(bitmap, 0, 0);
  bitmap.close();

  // Draw Heat point
  ctx.fillStyle = "black";
  ctx.textAlign = "end";
  for (let i = 0; i < points.length; i += 3) {
    const x = points[i];
    const y = points[i + 1];
    const heat = points[i + 2];
    const heatX = ((x - minX) / (maxX - minX)) * hw;
    const heatY = ((y - minY) / (maxY - minY)) * hh;

    ctx.fillRect(heatX - 2, heatY - 2, 4, 4);
    ctx.fillText(`${heat.toFixed(1)}`, heatX, heatY);
  }

  hmap.free();
}
