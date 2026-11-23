Status: cross-checked (manager, 2026-11-23)

# Running the inspiration as a test oracle

The warframe-damage.com page runs fully offline in headless Chromium (pre-installed Playwright).
`research/external/oracle/probe.js` (gitignored) loads it from the local copies in
`research/external/inspiration/` and intercepts:

- `/`, `/min.js`, `/jquery-min.js`, `/style-min.css`, `/data/json/enemies-min.json` → local files
- `/getWeapon.php?name=X` → `samples/weapon-X.json` (spaces → underscores). Other weapons must be fetched
  once from the live site (politely) and cached there.
- `/getImg.php` → a 1x1 data URI; every other host → 204

Usage: `node probe.js "https://warframe-damage.com/?p=<lz-string build>&n=1"`; it writes `document.body.innerText`
to `out.txt` after ~8 s.

Result for `research/test-cases/user-example-furis.json` (one random run):

| Layer | Attack | Random queue DPS | TTK |
|---|---|---|---|
| Armor | Normal Attack | 18,492 | not killed |
| Armor | Incarnon Form | 3,022,784 | 2.440 s |
| Health | Normal Attack | 50,773 | not killed |
| Health | Incarnon Form | 8,313,399 | 1.190 s |

Caveats: the queue is random (median of per-second buckets), so values vary between runs. To compare
against a deterministic engine, either enable "Calculate average values over several full firing/attack cycles",
run several times and take the median, or seed `Math.random` with `page.addInitScript`.
