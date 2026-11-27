"""Spec 07 E-7.0: one Module:Stances/data entry -> the readable JSON form of `ComboTable` (combo_tables.json).

Used to write the fixture's combo tables (Blind Justice; the stanceless Nikana, Tonfa, Dagger, Machete and Glaive
tables; the Telos Boltace UniqueToWeapon table) and by check_combo_tables.py to audit them.

E-7.0 steps as implemented: 1 pad missing / short arrays with the defaults of 07 §4.1 (Hits 1, Procs none, Types
direct, physical factors 1, StatusMultiplier 1); 2 a `Procs` map with holes becomes a list; 3 extra `Procs` names are
appended to the last entry's list (other over-long arrays are cut); 4 "" is no proc, a single name a one-element
list; 5 `Strike.phys` = factor - 1 in the order SLA, IMP, PUN (the `Strike.phys` contract); 6 each entry expands into
`Hits[e]` identical strikes. Step 0 (the pipeline's override table: High Noon Block input 1) concerns no table of the
fixture. Proc names map to status types by 03 table 4.2; crowd-control names go to `cc` (07 §4.1). An unquoted name
in the Lua source is an unset global and evaluates to nil, i.e. no proc (07 §4.1, Blind Justice Forward Block
input 3; lua_table.py).

Shared combos follow the module's `addSharedCombos` (Stances_data.lua L4-28): a stance or unique table that lacks one
of Heavy, Slide, Aerial, Wall, Finisher, Slam takes the weapon type's own combo, and that combo is marked
`FromWeapon` (`from_weapon = true`) in both tables.
"""
import lua_table as lua

STATUS = {'Impact': 'IMP', 'Puncture': 'PUN', 'Slash': 'SLA', 'Bleed': 'SLA', 'Heat': 'HEA', 'Cold': 'COL',
          'Electricity': 'ELE', 'Toxin': 'TOX', 'Blast': 'BLA', 'Radiation': 'RAD', 'Gas': 'GAS',
          'Magnetic': 'MAG', 'Viral': 'VIR', 'Corrosive': 'COR', 'Void': 'VOI', 'Tau': 'TAU',
          'Knockdown': 'KND', 'Knockback': 'IMP', 'Lifted': 'LIF', 'Microwave': 'MIC'}
CC = {'Ragdoll': 'ragdoll', 'Stagger': 'stagger', 'Big Stagger': 'big_stagger',
      'Stun': 'stun', 'Finisher': 'finisher', 'Disarmed': 'disarmed'}
TY = {'': 'plain', '360': 'spin360', 'Sweep': 'sweep', 'Thrust': 'thrust', 'Ranged': 'ranged', 'Slam': 'slam'}
KINDS = [('Neutral', 'neutral'), ('Forward', 'forward'), ('Block', 'block'), ('Forward Block', 'forward_block'),
         ('Slide', 'slide'), ('Aerial', 'aerial'), ('Wall', 'wall'), ('Finisher', 'finisher'), ('Slam', 'slam'),
         ('Heavy', 'heavy')]
# SHARED_COMBOS of Stances_data.lua L4.
SHARED = {'Heavy', 'Slide', 'Aerial', 'Wall', 'Finisher', 'Slam'}


def pad(a, n, d):
    a = list(a or [])
    return (a + [d] * n)[:n]


def strikes_of(inp, u):
    dmg = inp['Dmg']
    n = len(dmg)
    hits = pad(inp.get('Hits'), n, 1)
    procs = inp.get('Procs') or []
    if isinstance(procs, dict):  # step 2: map with holes
        procs = [procs.get(str(i + 1), procs.get(i + 1, '')) for i in range(n)]
    procs = list(procs)
    if len(procs) > n:  # step 3
        extra = procs[n:]
        procs = procs[:n]
        last = procs[n - 1]
        last = last if isinstance(last, list) else ([last] if last else [])
        procs[n - 1] = last + [x for x in extra if x]
    procs = pad(procs, n, '')
    types = pad(inp.get('Types'), n, '')
    mul = {t: pad(inp.get(f'{name}Multiplier'), n, 1) for t, name in (('SLA', 'Slash'), ('IMP', 'Impact'),
                                                                      ('PUN', 'Puncture'))}
    sm = pad(inp.get('StatusMultiplier'), n, 1)
    out = []
    for e in range(n):
        p = procs[e]
        p = p if isinstance(p, list) else ([p] if p else [])
        st = [STATUS[x] for x in p if x in STATUS]
        cc = [CC[x] for x in p if x in CC]
        unknown = [x for x in p if x and x not in STATUS and x not in CC]
        assert not unknown, unknown
        s = {'cc': cc, 'entry': e, 'input': u, 'mult': dmg[e] / 100, 'ty': TY[types[e] or ''],
             'phys': [float(round(mul[t][e] - 1, 10)) for t in ('SLA', 'IMP', 'PUN')],
             'procs': st, 'status_mult': float(sm[e])}
        for _ in range(hits[e]):
            out.append(dict(s))
    return out


def combo(c, wiki, key, prefix, from_weapon, names_out, entry_name):
    strikes = []
    for u, inp in enumerate(c['Attacks']):
        strikes += strikes_of(inp, u)
    nm = f'wfd:combo/{prefix}/{key}'
    names_out[nm] = c.get('Name') or f'{entry_name} {wiki}'
    return {'duration': c.get('Duration'), 'from_weapon': from_weapon, 'key': key, 'kind': key,
            'n_inputs': len(c['Attacks']), 'name': nm, 'strikes': strikes}


def table(lua_text, name, cls, names_out, combo_prefix, source=None):
    """The ComboTable of module entry `name` for weapon class `cls`.

    `source` is the table's `source` field (default `{"stanceless": cls}`: the entry is the weapon type's own table);
    for a stance or unique table pass `{"stance": …}` / `{"unique": {"name": …}}`, and shared combos the entry lacks
    are taken from its `WeaponType` entry. The table's `id` is -1 (fixture_io.save renumbers)."""
    t = lua.block(lua_text, name)
    own_type = source is None
    weapon = None if own_type else lua.block(lua_text, t['WeaponType'])
    combos = []
    for wiki, key in KINDS:
        c = t.get(wiki)
        if c:
            combos.append(combo(c, wiki, key, combo_prefix, own_type and wiki in SHARED, names_out, name))
        elif weapon is not None and wiki in SHARED and name != 'Ravenous Wraith':
            combos.append(combo(weapon[wiki], wiki, key, combo_prefix, True, names_out, t['WeaponType']))
    return {'combos': combos, 'id': -1, 'source': source or {'stanceless': cls}, 'weapon_type': cls}
