"""Audit: recompute every combo table of mini/combo_tables.json from the cited Module:Stances/data revision.

    python3 -I data/fixtures/tools/check_combo_tables.py [--external DIR]

DIR is the gitignored research/external directory (CLAUDE.md; docs/project/continuation.md §4 rebuilds it); default
$WFD_EXTERNAL, else research/external of the repository. The module is read from
DIR/cache/wiki-modules/raw/Stances_data.lua (Module:Stances/data, the revision manifest.json cites). Each table is
converted with stance_e70.py (07 E-7.0) and compared with the fixture, combo by combo, field by field, together with
the combo names of names/en.json. Exit status 1 on any difference.
"""
import argparse, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import stance_e70  # noqa: E402

MINI = os.path.join(HERE, '..', 'mini')
REPO = os.path.normpath(os.path.join(HERE, '..', '..', '..'))


def rd(p):
    with open(os.path.join(MINI, p), encoding='utf-8') as f:
        return json.load(f)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--external', default=os.environ.get('WFD_EXTERNAL') or os.path.join(REPO, 'research', 'external'))
    a = ap.parse_args()
    path = os.path.join(a.external, 'cache', 'wiki-modules', 'raw', 'Stances_data.lua')
    if not os.path.exists(path):
        sys.exit(f'{path} not found: rebuild research/external (docs/project/continuation.md §4) or pass --external')
    with open(path, encoding='utf-8') as f:
        lua_text = f.read()
    tables = rd('combo_tables.json')
    stances = rd('stances.json')
    items = [x for f in ['weapon_mod', 'weapon_arcane', 'frame_mod', 'aura', 'frame_arcane', 'shard', 'stance',
                         'incarnon', 'buff', 'custom'] for x in rd(f'items/{f}.json')]
    names = rd('names/en.json')
    bad = 0
    for t in tables:
        src = t['source']
        if 'stanceless' in src:
            entry, source = src['stanceless'].capitalize(), None
        elif 'stance' in src:
            entry, source = names[items[stances[src['stance']]['item']]['name']], src
        else:
            entry, source = src['unique']['name'], src
        prefix = t['combos'][0]['name'].split('/')[1]
        new_names = {}
        g = stance_e70.table(lua_text, entry, t['weapon_type'], new_names, prefix, source)
        g['id'] = t['id']
        if g == t and all(names.get(k) == v for k, v in new_names.items()):
            print(f'ok    table {t["id"]} ({entry}): {len(t["combos"])} combos')
            continue
        bad += 1
        print(f'DIFF  table {t["id"]} ({entry})')
        have = {c['key']: c for c in t['combos']}
        want = {c['key']: c for c in g['combos']}
        for k in sorted(set(have) | set(want)):
            if have.get(k) != want.get(k):
                fields = sorted(f for f in set(have.get(k, {})) | set(want.get(k, {}))
                                if have.get(k, {}).get(f) != want.get(k, {}).get(f))
                print(f'        combo {k}: {fields or "missing"}')
        for k, v in new_names.items():
            if names.get(k) != v:
                print(f'        name {k}: fixture {names.get(k)!r}, module {v!r}')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
