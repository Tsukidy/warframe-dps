// Bundle budget (frontend.md §12; work-breakdown.md T12 acceptance): the gzip size of the JavaScript of
// `dist/assets` must stay within the budget (600 kB at this stage). Lazy chunks count too, so the number is an upper
// bound on what any route loads.
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const assets = join(root, "dist/assets");
const BUDGET = Number(process.env.WFD_JS_BUDGET_KB ?? 600) * 1000;

let total = 0;
const rows = [];
for (const f of readdirSync(assets).filter((f) => f.endsWith(".js"))) {
  const gz = gzipSync(readFileSync(join(assets, f)), { level: 9 }).length;
  total += gz;
  rows.push([f, gz]);
}
rows.sort((a, b) => b[1] - a[1]);
for (const [f, gz] of rows) console.log(`${(gz / 1000).toFixed(1).padStart(8)} kB  ${f}`);
console.log(`${(total / 1000).toFixed(1).padStart(8)} kB  total JS (gzip), budget ${(BUDGET / 1000).toFixed(0)} kB`);
if (total > BUDGET) {
  console.error("JS bundle over budget");
  process.exit(1);
}
