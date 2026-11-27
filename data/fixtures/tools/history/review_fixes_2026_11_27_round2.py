"""T03 review fixes, second run (on top of d0b7dd0): the data edits of the second review.

Historical record (README.md "Tools"); not idempotent on a later commit. Run once on d0b7dd0's fixture:

    python3 -I data/fixtures/tools/history/review_fixes_2026_11_27_round2.py

- Haven Foray: the second line of the perk, "With Overshields: Increase Base Damage by +30", as a class (c)
  `while_overshields` Incarnon flat (04 §5.13 "Conditional Incarnon flat perks"; W:Furis_Incarnon_Genesis L50-55).
- rules.json: descriptors for T-33 (per-buff caster Strength; the Gorgon scenario's ally Roar at Strength 2.0) and
  T-37 (caster of the T-13 Roar), spec 00 §6.1.
- manifest.json: `spec_rev` and the spec source stamp follow the spec commit the records now follow (49c7dd3, the
  2026-11-27 spec update: D-067 amended, ruling C-4), `data_version` and `created_at` bumped; `coverage` is
  recomputed by fixture_io.save (coverage.py).
"""
import os, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
import fixture_io as fx  # noqa: E402

SPEC_REV = '49c7dd3'
b = fx.load()

hf = fx.item(b, 'wfd:perk/furis/haven_foray')
assert len(hf['effects']) == 1, 'already applied'
hf['effects'].append({
    'note': ('second line of the perk; adds to X (the page lists both lines, and its changelog L129 fixes "Haven '
             'Foray Incarnon perk not applying its bonus damage from Overshields")'),
    'scope': {'form': 'normal'}, 'stat': 'incarnon_flat', 'value': 30.0,
    'when': {'trigger': 'while_overshields'}})
hf['sources'] = ['W:Furis_Incarnon_Genesis L50-55, L129 (rev 2814461)', 'F:mods §9',
                 '04 §5.13 (conditional Incarnon flat perks: "+X while Overshields" class (c) while_overshields)',
                 'domain-model.md §16.10 (its example shows the first line only)']
hf['upstream_text'] = 'Increase Base Damage by +X.\nWith Overshields: Increase Base Damage by +30.'


def select(*vals):
    return {'kind': 'select', 'options': [{'label': f'opt.{v}', 'value': v} for v in vals]}


tg = b.rules['toggles']
assert not any(t['id'] in ('T-33', 'T-37') for t in tg), 'already applied'
tg += [
    {'advanced': True, 'control': select('manual', 'derive'), 'default': 'manual',
     'field': 'toggles.strength_mode|toggles.ability_duration|buffs.caster', 'group': 'scenario',
     'help': 'toggle.t33.help', 'id': 'T-33', 'label': 'toggle.t33.label'},
    {'advanced': False, 'control': select('self', 'ally'), 'default': 'ally', 'field': 'buffs.caster',
     'group': 'buffs', 'help': 'toggle.t37.help', 'id': 'T-37', 'label': 'toggle.t37.label'},
]
tg.sort(key=lambda t: int(t['id'][2:]))
fx.save(b)

man = fx.rd('manifest.json')
man['spec_rev'] = SPEC_REV
for s in man['sources']:
    if s['source'] == 'spec':
        s['revision'] = SPEC_REV
man['data_version'] = 'fixture-mini.2'
man['created_at'] = '2026-11-27T00:00:00Z'
man['curated_rev'] = 'fixture (T03, hand-written; data/fixtures/tools)'
fx.wr('manifest.json', man)
