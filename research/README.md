# Research (unfinished findings)

Everything in this directory is work in progress and may be wrong. Finished, cross-checked material moves to `docs/`.

- `findings/` — one file per topic. First line: `Status: draft | cross-checked | superseded`. Cite sources inline.
- `test-cases/` — builds with expected results. `user-example-furis.json` is the build from the warframe-damage.com link the project was started from (decoded from its `?p=` parameter).
- `external/` — gitignored third-party references. `inspiration/` holds the warframe-damage.com page, guide and JS bundle (`min.pretty.js` is the readable copy). `decode-inspiration-url.js` decodes a `?p=` build link (needs the `lz-string` npm package on `NODE_PATH`).
