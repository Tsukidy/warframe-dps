"""Fixture editing helper for data/fixtures/mini (README.md "Tools").

`load()` reads every file of the bundle and replaces dense-id cross references by symbols (`I:<uniqueName>` for items,
`W:` weapons, `S:` sets, `ST:` stances by their item, `CT:` combo tables by their source, `F:` frames), so records can
be added, removed and edited without tracking ids; `save()` re-sorts every record list by uniqueName, renumbers the
dense ids, resolves the symbols back, sorts `stable_keys.json` by key, recomputes `manifest.coverage`
(coverage.py) and writes the readable JSON form (2-space indent, sorted keys) that `tests/fixtures.rs` expects.
"""
import json, os, glob

# The bundle directory; WFD_FIXTURE_DIR overrides it (e.g. a scratch copy).
ROOT = os.environ.get('WFD_FIXTURE_DIR') or os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'mini')
WEAPON_FILES = ["primary", "secondary", "melee", "archgun", "archmelee", "exalted", "companion", "modular"]
ITEM_FILES = ["weapon_mod", "weapon_arcane", "frame_mod", "aura", "frame_arcane", "shard", "stance", "incarnon", "buff", "custom"]


def rd(p):
    return json.load(open(os.path.join(ROOT, p)))


def wr(p, o):
    open(os.path.join(ROOT, p), 'w').write(json.dumps(o, indent=2, sort_keys=True, ensure_ascii=False) + '\n')


def enemy_files():
    return sorted(os.path.relpath(p, ROOT) for p in glob.glob(os.path.join(ROOT, 'enemies/*.json')))


class B:
    pass


def load():
    b = B()
    b.weapons = {f: rd(f'weapons/{f}.json') for f in WEAPON_FILES}
    b.items = {f: rd(f'items/{f}.json') for f in ITEM_FILES}
    b.enemies = {f: rd(f) for f in enemy_files()}
    b.sets = rd('sets.json')
    b.stances = rd('stances.json')
    b.combo = rd('combo_tables.json')
    b.frames = rd('frames.json')
    b.modular = rd('modular.json')
    b.rules = rd('rules.json')
    b.names = rd('names/en.json')
    b.keys = rd('stable_keys.json')
    b.sources = None
    # symbolize
    W = [w['unique_name'] for f in WEAPON_FILES for w in b.weapons[f]]
    I = [x['unique_name'] for f in ITEM_FILES for x in b.items[f]]
    S = [s['unique_name'] for s in b.sets]
    ST = [I[s['item']] for s in b.stances]
    for t in b.combo:
        if 'stance' in t['source']:
            t['source']['stance'] = 'ST:' + ST[t['source']['stance']]
    CT = [json.dumps(t['source'], sort_keys=True) for t in b.combo]
    FR = [f['unique_name'] for f in b.frames]
    i = lambda n: 'I:' + I[n]
    for x in all_items(b):
        x['incompat'] = [i(n) for n in x['incompat']]
        if x.get('set') is not None:
            x['set'] = 'S:' + S[x['set']]
        k = x['kind']
        if isinstance(k, dict) and 'stance' in k:
            k['stance']['stance'] = 'ST:' + ST[k['stance']['stance']]
        c = x.get('compat') or {}
        for fld in ('weapons', 'magazine_exempt'):
            if c.get(fld):
                c[fld] = ['W:' + W[n] for n in c[fld]]
    for s in b.sets:
        s['members'] = [i(n) for n in s['members']]
        s['noncounting'] = [i(n) for n in s['noncounting']]
    for s in b.stances:
        s['item'] = i(s['item'])
        s['table'] = 'CT:' + CT[s['table']]
    for w in all_weapons(b):
        if w.get('incarnon'):
            w['incarnon']['tiers'] = [[i(n) for n in t] for t in w['incarnon']['tiers']]
        for a in w['attacks']:
            co = a.get('co')
            if co and co.get('off_with'):
                co['off_with'] = [i(n) for n in co['off_with']]
        m = w.get('melee')
        if m:
            m['stanceless'] = 'CT:' + CT[m['stanceless']]
            if m.get('unique') is not None:
                m['unique'] = 'CT:' + CT[m['unique']]
            if m.get('fixed_stance') is not None:
                m['fixed_stance'] = 'ST:' + ST[m['fixed_stance']]
    for c in b.modular['chambers']:
        c['weapon'] = 'W:' + W[c['weapon']]
    for f in b.frames:
        if f.get('variant_of') is not None:
            f['variant_of'] = 'F:' + FR[f['variant_of']]
        if f.get('helminth_host') is not None:
            f['helminth_host'] = i(f['helminth_host'])
        for a in f['abilities']:
            if a.get('self_buff') is not None:
                a['self_buff'] = i(a['self_buff'])
    return b


def all_items(b):
    return [x for f in ITEM_FILES for x in b.items[f]]


def all_weapons(b):
    return [x for f in WEAPON_FILES for x in b.weapons[f]]


def find(lst, un):
    for x in lst:
        if x['unique_name'] == un:
            return x
    raise KeyError(un)


def item(b, un):
    return find(all_items(b), un)


def weapon(b, un):
    return find(all_weapons(b), un)


def save(b):
    by_un = lambda x: x['unique_name']
    for f in WEAPON_FILES:
        b.weapons[f].sort(key=by_un)
    for f in ITEM_FILES:
        b.items[f].sort(key=by_un)
    for f in b.enemies:
        b.enemies[f].sort(key=by_un)
    b.sets.sort(key=by_un)
    b.frames.sort(key=by_un)
    W = {w['unique_name']: n for n, w in enumerate(all_weapons(b))}
    I = {x['unique_name']: n for n, x in enumerate(all_items(b))}
    for n, w in enumerate(all_weapons(b)):
        w['id'] = n
    for n, x in enumerate(all_items(b)):
        x['id'] = n
    for n, e in enumerate([e for f in sorted(b.enemies) for e in b.enemies[f]]):
        e['id'] = n
    S = {s['unique_name']: n for n, s in enumerate(b.sets)}
    for n, s in enumerate(b.sets):
        s['id'] = n
    # stances: keep order sorted by stance item uniqueName
    b.stances.sort(key=lambda s: s['item'])
    ST = {s['item'][2:]: n for n, s in enumerate(b.stances)}
    for n, s in enumerate(b.stances):
        s['id'] = n
    CT = {json.dumps(t['source'], sort_keys=True): n for n, t in enumerate(b.combo)}
    for n, t in enumerate(b.combo):
        t['id'] = n
    FR = {f['unique_name']: n for n, f in enumerate(b.frames)}
    for n, f in enumerate(b.frames):
        f['id'] = n

    def r(v):
        if not isinstance(v, str):
            raise ValueError(f'unsymbolized ref {v!r}')
        p, _, un = v.partition(':')
        return {'I': I, 'W': W, 'S': S, 'ST': ST, 'CT': CT, 'F': FR}[p][un]
    # unsymbolize
    for x in all_items(b):
        x['incompat'] = [r(v) for v in x['incompat']]
        if x.get('set') is not None:
            x['set'] = r(x['set'])
        k = x['kind']
        if isinstance(k, dict) and 'stance' in k:
            k['stance']['stance'] = r(k['stance']['stance'])
        c = x.get('compat') or {}
        for fld in ('weapons', 'magazine_exempt'):
            if c.get(fld):
                c[fld] = [r(v) for v in c[fld]]
    for s in b.sets:
        s['members'] = sorted(r(v) for v in s['members'])
        s['noncounting'] = sorted(r(v) for v in s['noncounting'])
    for s in b.stances:
        s['item'] = r(s['item'])
        s['table'] = r(s['table'])
    for t in b.combo:
        if 'stance' in t['source']:
            t['source']['stance'] = r(t['source']['stance'])
    for w in all_weapons(b):
        if w.get('incarnon'):
            w['incarnon']['tiers'] = [[r(v) for v in t] for t in w['incarnon']['tiers']]
        for a in w['attacks']:
            co = a.get('co')
            if co and co.get('off_with'):
                co['off_with'] = [r(v) for v in co['off_with']]
        m = w.get('melee')
        if m:
            m['stanceless'] = r(m['stanceless'])
            if m.get('unique') is not None:
                m['unique'] = r(m['unique'])
            if m.get('fixed_stance') is not None:
                m['fixed_stance'] = r(m['fixed_stance'])
    for c in b.modular['chambers']:
        c['weapon'] = r(c['weapon'])
    for f in b.frames:
        if f.get('variant_of') is not None:
            f['variant_of'] = r(f['variant_of'])
        if f.get('helminth_host') is not None:
            f['helminth_host'] = r(f['helminth_host'])
        for a in f['abilities']:
            if a.get('self_buff') is not None:
                a['self_buff'] = r(a['self_buff'])
    b.keys.sort(key=lambda kv: kv[0])
    for f in WEAPON_FILES:
        wr(f'weapons/{f}.json', b.weapons[f])
    for f in ITEM_FILES:
        wr(f'items/{f}.json', b.items[f])
    for f in b.enemies:
        wr(f, b.enemies[f])
    wr('sets.json', b.sets)
    wr('stances.json', b.stances)
    wr('combo_tables.json', b.combo)
    wr('frames.json', b.frames)
    wr('modular.json', b.modular)
    wr('rules.json', b.rules)
    wr('names/en.json', b.names)
    wr('stable_keys.json', b.keys)
    import coverage
    coverage.write(ROOT)
