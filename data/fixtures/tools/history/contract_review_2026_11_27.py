"""Contract review fixes (data area, 2026-11-27): the fixture edits of the whole-contract review, pass 1.

Historical record (README.md "Tools"); not idempotent. Run once on the fixture of ccr-659c593d-av1db8 at fc67d67:

    python3 -I data/fixtures/tools/history/contract_review_2026_11_27.py

Item ids are those of docs/project/inputs/contract-review-pass1.json and CONTRACTS.md "Contract review fixes (data)".

- C-37 Melee Doughty: the stat-registry row `doughty_rank` (b_r by rank; the crit stage computes 02 E-2.5a), verified.
- C-38 Theorem Demulcent: four effect lines, one per Residual element, each `value = { param = "<elem>_bonus" }`
  (0 … 1.80, default 0: a zero value holds no combine position, 01 §4.10.1), `when = in_zone`, position
  `warframe_arcane`; 04 §5.5 as the spec round worded it. Verified.
- DC-1 Melee Animosity: `per_event = "any_input"` (every light, slide, aerial or wall input).
- DC-14 Toxic Lash without `status_duration_add`; frames Saryn (passive SD +0.25, `Frame.passives`) and Rhino (Helminth
  host of Roar, DC-23 (e)).
- DC-16 `ability_source` on every buff (04 §4.2, §5.6, §5.7 "Source kind").
- DC-22 Piercing Navigator `default_count = 0`; Arcane Crepuscular "+3x" bound as `n2`.
- M-2 custom stat heatInheritFaction (`heat_inherit_faction`, 03 §10.3 F_inh). M-18: abilityCombo cites its T-11 alias.
- T03b-R6, DC-23, DC-19, API-12: rules.json rows become copies of the curated rule tables (data/curated/rules/*.toml):
  toggle descriptors, slot layouts (inc2-inc5 on primary and melee), target_vuln groups `dot = none` without type
  lists, the body-part fallback row, the Helminth restricted list and the Rhino host, the riven traits' keys and
  `vintage_companion`, Hemocyte's `mdps_frac`, `game_constants`.
- manifest.json: `spec_rev` fc67d67 (the spec the records now follow), `data_version` fixture-mini.3.
"""
import json
import os
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..'))
import fixture_io as fx  # noqa: E402

RULES = os.path.join(HERE, '..', '..', '..', 'curated', 'rules')
SPEC_REV = 'fc67d67'
ARCANE = 'wiki:Module:Arcane/data@2026-11-21T02:03:16Z'
WARFRAMES = 'wiki:Module:Warframes/data@2026-11-10T01:27:04Z'


def toml(name):
    with open(os.path.join(RULES, name), 'rb') as f:
        return tomllib.load(f)


b = fx.load()

# ---- C-37 Melee Doughty ----------------------------------------------------------------------------------------------
d = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/CritDamageForPunctureStatus')
assert d['effects'][0]['stat'] == 'cd_abs', 'already applied'
d['effects'] = [{
    'note': ('b_r = 0.5 + 0.1·rank (02 E-2.5a, 04 §5.4); the crit stage computes doughty = min(50, '
             'round1(10·(r_PUN/Σ_t r_t)·SC_w·b_r)) per instance and adds it to ΣCD_abs,i as its own term '
             '(CONTRACTS.md ruling C-37)'),
    'stat': 'doughty_rank',
    'value': {'max': 1.0, 'rank': {'table': [0.5, 0.6, 0.7, 0.8, 0.9, 1.0]}},
}]
d['status'] = 'verified'
d['sources'] = [f'{ARCANE}#Melee Doughty', '02 E-2.5a', '04 §5.4',
                'stat registry doughty_rank (CONTRACTS.md ruling C-37)', 'domain-model.md §16.13',
                'TV-02-14', 'TV-04-22', 'W:Melee_Doughty']

# ---- C-38 Theorem Demulcent ------------------------------------------------------------------------------------------
td = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/WeaponDamageOnResidualContact')
assert len(td['effects']) == 1, 'already applied'
scope = td['effects'][0]['scope']
elems = [('HEA', 'Residual Boils'), ('COL', 'Residual Malodor'), ('ELE', 'Residual Shock'), ('TOX', 'Residual Viremia')]
td['effects'] = [{
    'element': e,
    'note': (f'{zone}: the reached bonus v_{e} the user declares (0.12 per second in the zone, up to 15 stacks; '
             '04 §5.5); 0 holds no combine position (01 §4.10.1); position 10, sub-index = HCET index (E-1.8)'),
    'order': 'warframe_arcane',
    'scope': scope,
    'value': {'param': f'{e.lower()}_bonus'},
    'when': {'trigger': 'in_zone'},
} for e, zone in elems]
td['params'] = [{'default': 0.0, 'key': f'{e.lower()}_bonus', 'kind': 'percent', 'max': 1.8, 'min': 0.0}
                for e, _ in elems]
td['status'] = 'verified'
td['sources'] = [f'{ARCANE}#Theorem Demulcent',
                 '04 §5.5 (four per-element values v_HEA … v_TOX in [0, 1.80], default 0; CONTRACTS.md C-38)',
                 '01 §4.10.1 position 10', 'TV-01-38 (v_TOX = 1.80)', 'W:Theorem_Demulcent L6-25, L34-39']

# ---- DC-1 Melee Animosity --------------------------------------------------------------------------------------------
an = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/HeavyCritOnRegularMelee')
assert an['effects'][0]['params']['per_event'] == {'input': 'light'}, 'already applied'
an['effects'][0]['params']['per_event'] = 'any_input'
an['sources'] = [f'{ARCANE}#Melee Animosity', '04 §5.4 (one stack per light, slide, aerial or wall input)',
                 '07 §7.5', '04 §5.12', 'EC-07-28', 'domain-model.md §16.13']

# ---- DC-22 Arcane Crepuscular, Piercing Navigator --------------------------------------------------------------------
cr = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Utility/AbilityStrengthAndCritDamageWhenInvisible')
assert cr['effects'][0]['value']['bind'] == 'n2x', 'already applied'
cr['effects'][0]['value']['bind'] = 'n2'
cr['sources'] = [f'{ARCANE}#Arcane Crepuscular', '04 §5.5', '02 E-2.4',
                 'domain-model.md §15.2 ("+Nx" binds as nK)']
pn = fx.item(b, 'wfd:buff/piercing_navigator')
pn['effects'][0]['stacks']['default_count'] = 0
pn['sources'] = ['W:Piercing_Navigator L3, L27-29', '04 §5.3 (user count, default 0)', 'domain-model.md §16.13']

# ---- DC-14 Toxic Lash ------------------------------------------------------------------------------------------------
tl = fx.item(b, 'wfd:buff/toxic_lash')
assert tl['effects'][0]['params'].pop('status_duration_add') == 0.25, 'already applied'
tl['sources'] = ['W:Toxic_Lash', 'E-14', 'D-037',
                 "04 E-4.14 (Saryn's passive is the frame's: Frame.passives of /Lotus/Powersuits/Saryn/Saryn)",
                 'domain-model.md §16.8']

# ---- DC-16 ability_source (04 §4.2, §5.6, §5.7 "Source kind") ---------------------------------------------------------
ABILITY = {'ally_roar', 'eclipse', 'electric_shield', 'molecular_prime', 'nourish', 'piercing_navigator', 'roar',
           'sonar', 'terrify', 'toxic_lash', 'xatas_whisper'}
for x in b.items['buff']:
    slug = x['unique_name'].split('/', 1)[1]
    x['kind']['buff']['ability_source'] = slug in ABILITY

# ---- M-2 heatInheritFaction, M-18 abilityCombo ----------------------------------------------------------------------
hi = fx.item(b, 'wfd:custom/heatinherit')
hf = json.loads(json.dumps(hi))
hf.update({'key': 3726, 'name': 'wfd:custom/heatinheritfaction', 'unique_name': 'wfd:custom/heatinheritfaction',
           'effects': [{'stat': 'heat_inherit_faction', 'value': {'param': 'value'}}],
           'sources': ['04 §10', '00 §6.3 (ours, companion of heatInherit)', '03 §10.3 (F_inh)', 'TV-03-33']})
hf['kind']['custom_stat']['inspiration_key'] = 'heatInheritFaction'
b.items['custom'].append(hf)
b.names['wfd:custom/heatinheritfaction'] = 'Heat inherit faction'
b.keys.append([3726, 'wfd:custom/heatinheritfaction'])
ac = fx.item(b, 'wfd:custom/abilitycombo')
ac['sources'] = ['04 §10', '00 §6.3',
                 'domain-model.md §9.3 T-26 (a T-11 alias: resolve moves it into toggles.ability_combo; M-18)']

# ---- Frames: Saryn (DC-14), Rhino (DC-23 (e)) -------------------------------------------------------------------------
for f in b.frames:
    f.setdefault('passives', [])


def ability(un, buff=None):
    return {'name': un, 'self_buff': buff}


b.frames += [
    {'abilities': [ability('/Lotus/Powersuits/Rhino/Abilities/RhinoChargeAbility'),
                   ability('/Lotus/Powersuits/Rhino/Abilities/IronSkinAbility'),
                   ability('/Lotus/Powersuits/Rhino/Abilities/RhinoRoarAbility', 'I:wfd:buff/roar'),
                   ability('/Lotus/Powersuits/Rhino/Abilities/RhinoStompAbility')],
     'aura_polarities': ['madurai'], 'aura_slots': 1, 'exilus_polarity': None,
     'helminth_host': 'I:wfd:buff/roar', 'id': 0, 'image': 'Rhino_Thumb.png', 'key': 8002,
     'name': '/Lotus/Powersuits/Rhino/Rhino', 'passives': [], 'polarities': ['vazarin', 'vazarin'],
     'unique_name': '/Lotus/Powersuits/Rhino/Rhino', 'variant_of': None},
    {'abilities': [ability('/Lotus/Powersuits/Saryn/Abilities/PoisonAbility'),
                   ability('/Lotus/Powersuits/Saryn/Abilities/ShedAbility'),
                   ability('/Lotus/Powersuits/Saryn/Abilities/WeaponPoisonAbility', 'I:wfd:buff/toxic_lash'),
                   ability('/Lotus/Powersuits/Saryn/Abilities/ExplosiveDissolveAbility')],
     'aura_polarities': ['vazarin'], 'aura_slots': 1, 'exilus_polarity': None, 'helminth_host': None, 'id': 0,
     'image': 'Saryn_Thumb.png', 'key': 8003, 'name': '/Lotus/Powersuits/Saryn/Saryn',
     'passives': [{'stat': 'status_dur', 'value': 0.25}], 'polarities': ['naramon', 'vazarin'],
     'unique_name': '/Lotus/Powersuits/Saryn/Saryn', 'variant_of': None},
]
b.names.update({
    '/Lotus/Powersuits/Rhino/Rhino': 'Rhino',
    '/Lotus/Powersuits/Rhino/Abilities/RhinoChargeAbility': 'Rhino Charge',
    '/Lotus/Powersuits/Rhino/Abilities/IronSkinAbility': 'Iron Skin',
    '/Lotus/Powersuits/Rhino/Abilities/RhinoRoarAbility': 'Roar',
    '/Lotus/Powersuits/Rhino/Abilities/RhinoStompAbility': 'Rhino Stomp',
    '/Lotus/Powersuits/Saryn/Saryn': 'Saryn',
    '/Lotus/Powersuits/Saryn/Abilities/PoisonAbility': 'Spores',
    '/Lotus/Powersuits/Saryn/Abilities/ShedAbility': 'Molt',
    '/Lotus/Powersuits/Saryn/Abilities/WeaponPoisonAbility': 'Toxic Lash',
    '/Lotus/Powersuits/Saryn/Abilities/ExplosiveDissolveAbility': 'Miasma',
})
b.keys += [[8002, '/Lotus/Powersuits/Rhino/Rhino'], [8003, '/Lotus/Powersuits/Saryn/Saryn']]

# ---- rules.json: copies of the curated rule tables -------------------------------------------------------------------
R = b.rules

# Toggles (T03b-R6, DC-23 (b)): the descriptor of each id the fixture carries, as data/curated/rules/toggles.toml has it.
real_toggles = {t['id']: t for t in toml('toggles.toml')['toggle']}
R['toggles'] = [dict({'advanced': False, 'default': None}, **real_toggles[t['id']]) for t in R['toggles']]

# Multiplier groups (T03b-R6, DC-23 (a)): target_vuln groups are `dot = none` and carry no type list (every type).
for g in R['mult_groups']:
    if g['stage'] == 'target_vuln':
        g['dot'] = 'none'
        g['vuln'].pop('types', None)

# Slot layouts (DC-23 (c)): the curated rows of the four fixture classes (inc2-inc5 on primary and melee).
real_layouts = {l['class']: l for l in toml('slot_layout.toml')['layout']}
SLOT_DEFAULTS = {'arcane_family': None, 'element_order': None, 'fixed': False}
R['slot_layouts'] = [{'class': l['class'], 'source': real_layouts[l['class']]['source'],
                      'slots': [dict(SLOT_DEFAULTS, **s) for s in real_layouts[l['class']]['slots']]}
                     for l in R['slot_layouts']]

# Body parts (DC-23 (d)): the fallback row of every other faction.
fallback = [r for r in toml('body_parts.toml')['rules']['faction_heads'] if not r['factions']]
assert len(fallback) == 1 and not any(not r['factions'] for r in R['body_parts']['faction_heads'])
R['body_parts']['faction_heads'].append(fallback[0])

# Helminth (DC-23 (e)): the restricted list R of 09 E-9.32 and the host row of the fixture's Rhino.
h = toml('helminth.toml')['rules']
R['helminth'] = {'hosts': [x for x in h['hosts'] if x['frame'] == '/Lotus/Powersuits/Rhino/Rhino'],
                 'restricted': sorted(h['restricted']),
                 'sources': ['09 E-9.32 (R complete; Host: the Rhino row, the fixture frame that is a host)']}

# Riven traits (API-12): keys, PE tags and the companion vintage flag of the curated rows (recipes stay out: the
# fixture has no recipe traits).
real_traits = {t['id']: t for t in toml('riven_traits.toml')['trait']}
for t in R['riven_traits']:
    rt = real_traits[t['id']]
    for k in ('inspiration_key', 'pe_tag', 'vintage_companion'):
        if k in rt:
            t[k] = rt[k]

# Special DR definitions: Hemocyte's documented MDPS fraction (05 §15.2, stored, not applied).
real_dr = {x['id']: x for x in toml('special_dr.toml')['dr']}
for x in R['special_dr']:
    x['attenuation'] = real_dr[x['id']].get('attenuation')

# Game constants (DC-19): the curated table.
R['game_constants'] = {c['key']: c['value'] for c in toml('game_constants.toml')['constant']}

fx.save(b)

man = fx.rd('manifest.json')
man['spec_rev'] = SPEC_REV
for s in man['sources']:
    if s['source'] == 'spec':
        s['revision'] = SPEC_REV
man['data_version'] = 'fixture-mini.3'
fx.wr('manifest.json', man)
