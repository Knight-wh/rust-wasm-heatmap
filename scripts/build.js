const fs = require("fs");
const path = require("path");
const packageJsonPath = path.join(__dirname, "../pkg/package.json");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));

packageJson.homepage = "https://github.com/Knight-wh/rust-wasm-heatmap";
packageJson.files.push("rust_wasm_heatmap_bg.wasm.d.ts");

fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));
