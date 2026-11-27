"""`manifest.coverage` of the mini bundle: item count per item kind per `EffectStatus` (bundle.rs `Manifest.coverage`).

    python3 -I data/fixtures/tools/coverage.py [--write]

Without `--write` it prints the computed map and exits 1 when manifest.json differs; with `--write` it stores it.
`tests/fixtures.rs` (`manifest_coverage_counts_the_items`) checks the same equality. Kind slugs are
`ItemKind::slug()` (item.rs); extension files (`extra/*.json`) are counted too (README.md "Extension files" rule 5).
"""
import glob, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ITEM_FILES = ['weapon_mod', 'weapon_arcane', 'frame_mod', 'aura', 'frame_arcane', 'shard', 'stance', 'incarnon',
              'buff', 'custom']


def kind_slug(kind):
    """`ItemKind::slug()`: the serde tag of the kind (`aura` and `frame_arcane` are unit variants, i.e. strings)."""
    return kind if isinstance(kind, str) else next(iter(kind))


def compute(root):
    items = []
    for f in ITEM_FILES:
        with open(os.path.join(root, 'items', f + '.json'), encoding='utf-8') as fh:
            items += json.load(fh)
    for p in sorted(glob.glob(os.path.join(root, 'extra', '*.json'))):
        with open(p, encoding='utf-8') as fh:
            items += json.load(fh).get('items', [])
    cov = {}
    for x in items:
        k = cov.setdefault(kind_slug(x['kind']), {})
        k[x['status']] = k.get(x['status'], 0) + 1
    return {k: dict(sorted(v.items())) for k, v in sorted(cov.items())}


def write(root):
    p = os.path.join(root, 'manifest.json')
    with open(p, encoding='utf-8') as fh:
        man = json.load(fh)
    man['coverage'] = compute(root)
    with open(p, 'w', encoding='utf-8') as fh:
        fh.write(json.dumps(man, indent=2, sort_keys=True, ensure_ascii=False) + '\n')


def main():
    root = os.environ.get('WFD_FIXTURE_DIR') or os.path.join(HERE, '..', 'mini')
    if '--write' in sys.argv[1:]:
        write(root)
        return
    with open(os.path.join(root, 'manifest.json'), encoding='utf-8') as fh:
        have = json.load(fh)['coverage']
    want = compute(root)
    print(json.dumps(want, indent=2))
    if have != want:
        sys.exit('manifest.json coverage differs from the item files (run with --write)')


if __name__ == '__main__':
    main()
