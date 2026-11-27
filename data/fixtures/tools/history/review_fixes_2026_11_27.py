"""T03 review fixes, first run (commit d0b7dd0 "T03: fix the 15 review issues and the T01d fixture items").

Historical record of how the JSON of that commit was produced (README.md "Tools"): it restores the fixture JSON of
the commit before it (BASE, e3a0049), applies every data edit of the fix run and saves through fixture_io.py.
SOURCES.md, COVERAGE.md and README.md were edited by hand from the rows it prints. Running it again on a later
commit discards every fixture change made after BASE; use it to audit or reproduce that commit only:

    WFD_EXTERNAL=<research/external> python3 -I data/fixtures/tools/history/review_fixes_2026_11_27.py
    git diff d0b7dd0 -- data/fixtures   # only manifest.json coverage differs (save() now recomputes it)

It needs the cached Module:Stances/data (research/external/cache/wiki-modules/raw/Stances_data.lua) for the Machete
and Glaive tables.
"""
import copy, json, os, subprocess, sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
import fixture_io as fx, stance_e70 as stance  # noqa: E402

BASE = 'e3a0049'
WT = subprocess.run(['git', '-C', os.path.dirname(os.path.abspath(__file__)), 'rev-parse', '--show-toplevel'],
                    capture_output=True, text=True, check=True).stdout.strip()
RAW = os.path.join(os.environ.get('WFD_EXTERNAL') or os.path.join(WT, 'research', 'external'),
                   'cache', 'wiki-modules', 'raw')
files = subprocess.run(['git', '-C', WT, 'ls-tree', '-r', '--name-only', BASE, 'data/fixtures/'], capture_output=True,
                       text=True, check=True).stdout.split()
files = [f for f in files if f.endswith('.json')]
subprocess.run(['git', '-C', WT, 'checkout', BASE, '--'] + files, check=True)
for extra in ['data/fixtures/scenarios/gorgon_heavy_gunner_l150_strip_viral.json']:
    p = os.path.join(WT, extra)
    if os.path.exists(p):
        os.remove(p)

b = fx.load()
I = lambda un: 'I:' + un
MODS = 'wiki:Module:Mods/data@2026-11-01T23:35:48Z#'
ARC = 'wiki:Module:Arcane/data@2026-11-21T02:03:16Z#'
SOURCES = []  # (section, row) for SOURCES.md, printed at the end


def new_item(file, template, un, key, name, **over):
    t = copy.deepcopy(fx.item(b, template))
    t.update(unique_name=un, name=un, key=key, id=-1, incompat=[], set=None, inspiration_name=None,
             upstream_text=None, image=None, params=[])
    t.update(over)
    b.items[file].append(t)
    b.keys.append([key, un])
    b.names[un] = name
    SOURCES.append(('item', f"| {key} | `{un}` | {name} | {t['status']} | {'; '.join(t['sources'])} |"))
    return t


# ---------------------------------------------------------------------------------------------------- issue 1
for tab in b.combo:
    for c in tab['combos']:
        for s in c['strikes']:
            o = s['phys']
            s['phys'] = [o[2], o[0], o[1]]  # IMP, PUN, SLA -> SLA, IMP, PUN (Strike.phys contract)

# ---------------------------------------------------------------------------------------------------- issue 2
HF = 'wfd:perk/furis/haven_foray'
FURIS_SRC = ['W:Furis_Incarnon_Genesis', 'research/external/cache/wiki-modules/Furis_Incarnon_Genesis.wiki rev 2814461']


def perk(un, key, name, tier, index, effects, status, extra_src=(), upstream=None):
    return new_item('incarnon', HF, un, key, name, effects=effects, status=status,
                    kind={'incarnon_perk': {'family': 0, 'index': index, 'tier': tier}},
                    sources=FURIS_SRC + list(extra_src), upstream_text=upstream)


perk('wfd:perk/furis/stormburst', 3503, 'Stormburst', 2, 1, [
    {'scope': {'form': 'normal'}, 'stat': 'incarnon_flat', 'value': {'variants': {'furis': 28.0, 'mk1_furis': 34.0}}},
    {'note': 'UNVERIFIED: stack timer kind; the gain condition (hit on a target with Electricity status) is written as '
             'an on-hit trigger with a target predicate',
     'per': ['stack'], 'stacks': {'duration': 2.0, 'max': 3}, 'stat': 'multishot', 'value': 0.4,
     'when': {'all': [{'trigger': 'on_hit'}, {'target': {'has_status': 'ELE'}}]}}],
     'draft', ['04 §4.2'],
     'Increase Base Damage by +X. On hitting enemies affected by Electricity Status: +0.4 Multishot for 2s. Stacks up to 3x.')
perk('wfd:perk/furis/practiced_grip', 3504, 'Practiced Grip', 3, 0,
     [{'stat': 'display.accuracy', 'value': 0.5}], 'no_damage_effect', [], 'Increase Accuracy by +50%')
perk('wfd:perk/furis/executioners_fortune', 3505, "Executioner's Fortune", 3, 2, [], 'unmodelled',
     ['06 §5.6 (instant reload has no encoding in the fixture)'], 'On Headshot: 10% chance for Instant Reload')
perk('wfd:perk/furis/headcracker', 3506, 'Headcracker', 4, 0, [
    {'note': 'UNVERIFIED: stack timer kind (04 §4.2)', 'per': ['stack'],
     'stacks': {'duration': 2.0, 'max': 10}, 'stat': 'fire_rate', 'value': 0.05,
     'when': {'chance': 0.5, 'trigger': 'on_headshot'}}], 'draft', ['04 §4.2'],
     'On Headshot: +5% Fire Rate for 2s. Stacks up to 10x. (50% chance of activating)')
perk('wfd:perk/furis/elemental_balance', 3507, 'Elemental Balance', 4, 2, [
    {'stat': 'sc_base_add', 'value': {'variants': {'furis': 0.28, 'mk1_furis': 0.39}}}], 'verified',
     ['02 §4.6 SC0+ (Incarnon base status chance per projectile)', '09 EC-09-52'],
     'Increase Base Status Chance by +X%.')
new_item('incarnon', HF, 'wfd:perk/braton/survivors_edge', 3515, "Survivor's Edge", status='verified',
         kind={'incarnon_perk': {'family': 4, 'index': 2, 'tier': 4}},
         effects=[{'stat': 'cc_base_add', 'value': {'variants': {'braton': 0.10, 'braton_prime': 0.10,
                                                                  'braton_vandal': 0.10, 'mk1_braton': 0.12}}},
                  {'stat': 'sc_base_add', 'value': {'variants': {'braton': 0.12, 'braton_prime': 0.14,
                                                                  'braton_vandal': 0.12, 'mk1_braton': 0.12}}}],
         sources=['W:Braton_Incarnon_Genesis L128-135 (rev 2824593)', '02 §4.2 CC0+ (Incarnon base crit chance)',
                  '02 §4.6 SC0+'],
         upstream_text='Increase Base Critical Chance by +X. Increase Base Status Chance by +Y.')
furis = fx.weapon(b, '/Lotus/Weapons/Tenno/Pistol/AutoPistol')
P = 'wfd:perk/furis/'
furis['incarnon']['tiers'] = [
    [],
    [I(P + 'haven_foray'), I(P + 'stormburst')],
    [I(P + 'practiced_grip'), I(P + 'extended_volley'), I(P + 'executioners_fortune')],
    [I(P + 'headcracker'), I(P + 'prelude_of_might'), I(P + 'elemental_balance')],
    [],
]

# ---------------------------------------------------------------------------------------------------- issue 3
rub = fx.weapon(b, '/Lotus/Weapons/Tenno/LongGuns/FiveShotSniper/FiveShotSniper')
for z, txt in zip(rub['zoom'], ['3.5x (+35% Critical Multiplier)', '6x (+50% Critical Multiplier)']):
    for e in z['effects']:
        e['stat'] = 'cd_rel'
        e['note'] = (f'wiki Zoom "{txt}"; sniper zoom crit damage is ΣCD_rel (02 §4.3 bucket table; EC-02-17: '
                     'only Lanka\'s zoom is flat)')
rub['provenance']['sources'].append({'group': 'spec', 'revision': '-',
                                     'source': '02 §4.3 ΣCD_rel (sniper zoom crit damage), EC-02-17'})

# ---------------------------------------------------------------------------------------------------- issue 4
sd = fx.item(b, 'wfd:perk/phenmor/spiteful_defilement')
sd['effects'][0]['stat'] = 'cd_abs'
sd['sources'] = ['W:Phenmor L133-138', 'D-067 (amended 2026-11-24)', '02 §4.3 ΣCD_abs', '04 §5.12 (OQ-406 closed)',
                 'TV-02-01']

# ---------------------------------------------------------------------------------------------------- issue 5
dty = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/CritDamageForPunctureStatus')
dty['effects'] = [{
    'note': ('02 E-2.5a: doughty = min(50, round1((r_PUN/Σr)·100·SC_w·0.1·d_rank)); value 10·b_r with b_r = '
             '0.5 + 0.1·rank (04 §5.4). DSL gap (CONTRACTS.md, T03 fix): `sc_of.PUN` is SC·w_PUN of E-9, not the '
             'unquantized modded Puncture share times the weapon-side SC; no round1, no cap 50, no scope for the '
             'Puncture-bearing part only. Draft until the DSL can express E-2.5a'),
    'per': [{'resolved': {'curve': {'pow': {'exp': 1.0, 'scale': 1.0}}, 'stat': 'sc_of.PUN'}}],
    'stat': 'cd_abs',
    'value': {'max': 10.0, 'rank': {'table': [5.0, 6.0, 7.0, 8.0, 9.0, 10.0]}}}]
dty['status'] = 'draft'
dty['sources'] = [ARC + 'Melee Doughty', '02 E-2.5a', '02 §4.3 ΣCD_abs (OQ-209)', '04 §5.4', 'TV-02-14',
                  'TV-04-22', 'W:Melee_Doughty']

# ---------------------------------------------------------------------------------------------------- issue 6
dh = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnNoMelee')
dh['effects'][0]['stacks']['stack_timer'] = 'shared_lose_one'
del dh['effects'][0]['note']
dh['sources'] = [ARC + 'Primary Deadhead', 'domain-model.md §16.4', 'F:addmul §5',
                 '04 §4.2 stack-timer table (shared_lose_one, 24 s, full decay 72 s)', '04 §5.4']

# ---------------------------------------------------------------------------------------------------- issue 8
ma = fx.item(b, 'wfd:buff/magus_accelerant')
ma['effects'][0]['stacks'] = {'duration': 0.0, 'max': 1, 'unbounded': True}
ma['effects'][0]['note'] = ('count of applications, no limit (04 §5.5; default 1, ruling C-4): the Ignis Wraith example '
                            'of W:Magus_Accelerant uses 3')
ma['sources'] = [ARC + 'Magus Accelerant', '04 §5.5', '04 §5.7 (add group, HEA, HP)', 'domain-model.md §16.13',
                 'CONTRACTS.md ruling C-4 (2026-11-27)']

# ---------------------------------------------------------------------------------------------------- issue 13
pr = fx.item(b, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamagePerAmmoOnReload')
pr['effects'][0]['stacks']['duration_rank'] = {'table': [3.0, 4.0, 5.0, 6.0, 8.0, 10.0]}
pr['sources'].append('W:Primary_Plated_Round (duration by rank 3 / 4 / 5 / 6 / 8 / 10 s, 04 §5.4 row)')

# ---------------------------------------------------------------------------------------------------- T01d items
la = fx.item(b, 'wfd:perk/laetum/overwhelming_attrition')
la['effects'] = [{'per': ['stack'], 'stacks': {'duration': 10.0, 'max': 3, 'stack_timer': 'shared_lose_one'},
                  'stat': 'base_dmg', 'value': 4.0, 'when': {'hit_unit': 'pellet', 'trigger': 'on_plain_hit'}}]
la['sources'] = ['W:Laetum L145-149', '04 §5.12', '04 §4.1 E-4.9 (on_plain_hit)', 'D-410', 'TV-04-47',
                 'domain-model.md §16.13']
for g in b.rules['mult_groups']:
    if g['id'] == 'sonar':
        g['dot'] = 'inherit'
        g['sources'] = ['04 §3.2 (not in G: sonar_mult)', '02 E-2.10', 'D-095 (Part_dot = Part: DoT inherits M_son)',
                        '02 OQ-202 (closed)']
for lay in b.rules['slot_layouts']:
    for s in lay['slots']:
        if lay['class'] != 'warframe' and (s['id'] == 'arcane0' or s['id'].startswith('inc')):
            s['element_order'] = 9  # 01 §4.10.1 position 9 (E-1.8; domain-model.md §13)
    lay['source'] = lay['source'] + '; 01 §4.10.1 position 9 for arcane0 / inc<t> (CONTRACTS.md T01d)' \
        if lay['class'] != 'warframe' else lay['source']

ELEM_PARAM = {'choices': [{'value': 'HEA'}, {'value': 'COL'}, {'value': 'ELE'}, {'value': 'TOX'}],
              'key': 'element', 'kind': 'choice'}
ARACHNE = '/Lotus/Upgrades/CosmeticEnhancers/Utility/GolemArcaneBonusDamageOnWallLatch'
new_item('frame_arcane', ARACHNE, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/WeaponDamageOnResidualContact', 3413,
         'Theorem Demulcent', status='draft', inspiration_name='Theorem Demulcent',
         effects=[{'element': {'param': 'element'}, 'order': 'warframe_arcane', 'per': ['stack'],
                   'note': ('combining primary at position 10 (01 §4.10.1, E-1.8; sub-index = HCET index). One '
                            'entry holds one Residual element; several zones need several entries (each keeps its '
                            'own stacks, 04 §5.5). The default TOX serves TV-01-38: 04 §5.5 gives no default type'),
                   'scope': {'weapon': {'slots': ['primary', 'secondary', 'melee']}},
                   'stacks': {'duration': 20.0, 'max': 15}, 'value': {'max': 0.12},
                   'when': {'trigger': 'in_zone'}}],
         params=[dict(ELEM_PARAM, default='TOX')],
         sources=[ARC + 'Theorem Demulcent', '04 §5.5', '01 §4.10.1 position 10', 'TV-01-38', 'W:Theorem_Demulcent L34-39'],
         upstream_text=('Standing in a zone created by a Residual Arcane increases weapon damage by 12%/s stacking up '
                        'to 15x. Effect persists for 20s upon leaving the zone.'))
new_item('frame_arcane', ARACHNE, '/Lotus/Upgrades/CosmeticEnhancers/Offensive/OrbsOnResidualContact', 3414,
         'Theorem Contagion', status='verified', inspiration_name='Theorem Contagion',
         effects=[{'stacks': {'duration': 6.0, 'max': 1}, 'stat': 'vuln_add', 'value': {'max': 2.0},
                   'vuln': {'types': {'param': 'element'}}, 'when': {'trigger': 'in_zone'}}],
         params=[dict(ELEM_PARAM, default='HEA')],
         sources=[ARC + 'Theorem Contagion', '04 §5.5', '04 §5.7 (add group, the orb\'s type, HP)',
                  'domain-model.md §16.13'],
         upstream_text=('Standing in a zone created by a Residual Arcane creates a globe that orbits the player every '
                        '2s. The globes will strike the nearest enemy within 15m dealing 150 damage increasing their '
                        'vulnerability to the globe\'s damage type by 200% for 6s.'))

# ---------------------------------------------------------------------------------------------------- issue 7
# Terrify (strip_ability)
new_item('buff', 'wfd:buff/ally_roar', 'wfd:buff/terrify', 3615, 'Terrify', status='verified',
         effects=[{'per': ['strength'], 'stat': 'strip_ability', 'value': 0.6}],
         kind={'buff': {'group': 'ability', 'inspiration_key': None, 'strength_scaled': True, 'subsumable': True,
                        'toggle': None}},
         sources=['W:Terrify', '04 §5.7 armor strips (Terrify 0.60 × Str)', '05 E-5.10'])
GM = '/Lotus/Upgrades/Mods/Sets/Gladiator/MeleeGladiatorMightMod'
compat = lambda cats, slot, **kw: dict({'categories': cats, 'exalt_req': [], 'exalted_lift': 'none',
                                        'forbid_tags': [], 'magazine_exempt': [], 'min_base_magazine': None,
                                        'slot_type': slot, 'tags': [], 'trigger': None, 'weapons': []}, **kw)
# charge_rate, ammo_eff
new_item('weapon_mod', GM, '/Lotus/Upgrades/Mods/Archwing/Rifle/ArchwingRifleChargeSpeedMod', 3352, 'Shell Rush',
         status='verified', inspiration_name=None, max_rank=3, polarity='vazarin', rarity='uncommon', base_drain=4,
         compat=compat([], 'arch_gun'), effects=[{'stat': 'charge_rate', 'value': {'bind': 'n1%', 'max': 0.5}}],
         sources=[MODS + 'Shell Rush', '06 §5.1 (charge rate)'], upstream_text='+50% Charge Rate')
new_item('weapon_mod', GM, '/Lotus/Upgrades/Mods/Pistol/Event/AkstilettoEfficiencyWhileAirborneMod', 3353,
         'Zazvat-Kar', status='verified', max_rank=5, polarity='naramon', rarity='rare', base_drain=2,
         kind={'weapon_mod': {'augment_of': None, 'exilus': False}},
         compat=compat([], 'secondary'),
         effects=[{'stat': 'ammo_eff', 'value': {'bind': 'n1%', 'max': 0.75}, 'when': {'trigger': 'while_airborne'}}],
         sources=[MODS + 'Zazvat-Kar', '06 §5.6 (ammo efficiency ≤ 0.75)', '04 §4.1 while_airborne'],
         upstream_text='+75% Ammo Efficiency while Airborne')
# combo_dur (percent combo duration, 07 E-7.12)
new_item('weapon_mod', GM, '/Lotus/Upgrades/Mods/Melee/DualStat/CorruptedHeavyDamageChargeSpeedMod', 3349,
         'Corrupt Charge', status='verified', inspiration_name='Corrupt Charge', max_rank=3, polarity='madurai',
         rarity='rare', base_drain=4, compat=compat([6], 'melee'),
         effects=[{'stat': 'initial_combo', 'value': {'bind': 'n1', 'max': 30.0}},
                  {'stat': 'combo_dur', 'value': {'bind': 'n2%', 'max': -0.5}}],
         sources=[MODS + 'Corrupt Charge', '07 E-7.12 (combo duration percent)', '07 E-7.15 (Initial Combo)'],
         upstream_text='+30 Initial Combo\n-50% Combo Duration')
# Sacrificial set (member scale)
SAC = '/Lotus/Upgrades/Mods/Sets/Sacrifice/'
sac_note = ('Sentient part (x1.33 vs Sentients) omitted: the mini rules have no Sentient faction and the rank '
            'columns are UNVERIFIED (04 §5.13, OQ-415)')
new_item('weapon_mod', GM, SAC + 'MeleeSacrificeModB', 3346, 'Sacrificial Steel', status='draft',
         inspiration_name='Sacrificial Steel', max_rank=10, polarity='umbra', rarity='legendary', base_drain=6,
         set='S:' + SAC + 'SacrificeSetMod',
         effects=[{'note': sac_note, 'scope_factor': [{'factor': 2.0, 'when': {'attacks': {'any_of': ['heavy']}}}],
                   'stat': 'cc_rel', 'value': {'bind': 'n1%', 'max': 2.2}}],
         sources=[MODS + 'Sacrificial Steel', '04 §9 (Sacrificial ×1.25)', '04 §5.13', '09 EC-09-26'],
         upstream_text='+220% Critical Chance (x2 for Heavy Attacks)\nx1.33 Damage to Sentients')
new_item('weapon_mod', GM, SAC + 'MeleeSacrificeModA', 3347, 'Sacrificial Pressure', status='draft',
         inspiration_name='Sacrificial Pressure', max_rank=10, polarity='umbra', rarity='legendary', base_drain=6,
         set='S:' + SAC + 'SacrificeSetMod',
         effects=[{'note': sac_note, 'scope': {'weapon': {'slots': ['melee']}}, 'stat': 'base_dmg',
                   'value': {'bind': 'n1%', 'max': 1.1}}],
         sources=[MODS + 'Sacrificial Pressure', '04 §9 (Sacrificial ×1.25)', '04 §5.13', '09 EC-09-26'],
         upstream_text='+110% Melee Damage\nx1.33 Damage to Sentients')
for un in (SAC + 'MeleeSacrificeModB', SAC + 'MeleeSacrificeModA'):
    fx.item(b, un)['incompat'] = []
b.sets.append({'bonus': [], 'id': -1, 'member_scale': [1.0, 1.25],
               'members': [I(SAC + 'MeleeSacrificeModA'), I(SAC + 'MeleeSacrificeModB')],
               'name': SAC + 'SacrificeSetMod', 'noncounting': [], 'unique_name': SAC + 'SacrificeSetMod'})
b.names[SAC + 'SacrificeSetMod'] = 'Sacrificial Set'
# Nira's set (slam multiplier group)
NIRA = '/Lotus/Upgrades/Mods/Sets/Nira/'
b.rules['tags'].append({'id': len(b.rules['tags']), 'name': 'WHIPS_STANCE'})
new_item('weapon_mod', GM, NIRA + 'NiraMeleeMod', 3348, "Nira's Contempt", status='verified',
         inspiration_name="Nira's Contempt", max_rank=5, polarity='madurai', rarity='rare', base_drain=4,
         set='S:' + NIRA + 'NiraSetMod',
         compat=compat([], 'melee', tags=[len(b.rules['tags']) - 1], forbid_tags=[7]),
         effects=[{'scope': {'weapon': {'slots': ['melee']}}, 'stat': 'base_dmg', 'value': {'bind': 'n1%', 'max': 0.9}},
                  {'stat': 'sc_rel', 'value': {'bind': 'n2%', 'max': 0.6}}],
         sources=[MODS + "Nira's Contempt", "04 §9 (Nira's set)"], upstream_text='+90% Melee Damage\n+60% Status Chance')
VIG_FM = '/Lotus/Upgrades/Mods/Sets/Vigilante/WarframeVigilantePursuitMod'
new_item('frame_mod', VIG_FM, NIRA + 'NiraWarframeMod', 3326, "Nira's Hatred", status='no_damage_effect',
         inspiration_name=None, max_rank=5, polarity='vazarin', rarity='uncommon', base_drain=4,
         set='S:' + NIRA + 'NiraSetMod', effects=[],
         sources=[MODS + "Nira's Hatred"], upstream_text='+35% Health\n+15% Ability Duration')
new_item('frame_mod', VIG_FM, NIRA + 'NiraExilusMod', 3327, "Nira's Anguish", status='no_damage_effect',
         inspiration_name=None, max_rank=5, polarity='vazarin', rarity='common', base_drain=4,
         kind={'frame_mod': {'exilus': True}},
         set='S:' + NIRA + 'NiraSetMod', effects=[],
         sources=[MODS + "Nira's Anguish"], upstream_text='+15% to Parkour Velocity\n+15% Aim Glide/Wall Latch Duration')
b.sets.append({'bonus': [{'mult': 'slam', 'note': 'Slam Damage +1.50 per piece since U39 (was +1.00; the module text '
                          'still says +100%)', 'per': ['set_piece'],
                          'scope': {'attacks': {'any_of': ['slam', 'heavy_slam']}}, 'value': 1.5}],
               'id': -1, 'member_scale': None,
               'members': [I(NIRA + 'NiraMeleeMod'), I(NIRA + 'NiraWarframeMod'), I(NIRA + 'NiraExilusMod')],
               'name': NIRA + 'NiraSetMod', 'noncounting': [], 'unique_name': NIRA + 'NiraSetMod'})
b.names[NIRA + 'NiraSetMod'] = 'Nira Set'
b.rules['mult_groups'].append({'dot': 'inherit', 'id': 'slam', 'sources': [
    '04 §3.2 Slam Damage (1 + Σ on slams)', '07 §7.4 M_slam', 'W:Melee §Slam Damage'], 'stage': 'G',
    'twice_on_two_bucket': False})

# Weapons -------------------------------------------------------------------------------------------------------
STANCES = open(os.path.join(RAW, 'Stances_data.lua')).read()


def dvec(**kw):
    order = ['IMP', 'PUN', 'SLA', 'HEA', 'COL', 'ELE', 'TOX', 'BLA', 'RAD', 'GAS', 'MAG', 'VIR', 'COR', 'VOI', 'TAU',
             'TRU', 'CIN', 'FIN']
    return [float(kw.get(t, 0.0)) for t in order]


nik = fx.weapon(b, '/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana')


def attack(tmpl, un, idx, slug, name, **over):
    a = copy.deepcopy(tmpl)
    a.update(key={'idx': idx, 'slug': slug}, name=f'{un}#{slug}')
    a.update(over)
    b.names[f'{un}#{slug}'] = name
    return a


def add_table(lua_name, cls):
    t = stance.table(STANCES, lua_name, cls, b.names, cls)
    b.combo.append(t)
    return 'CT:' + json.dumps(t['source'], sort_keys=True)


# Machete (sigma != 1, TV-07-10)
MUN = '/Lotus/Weapons/Grineer/Melee/GrineerMachetteAndCleaver/Machete'
m = copy.deepcopy(nik)
na, sl, hs, hv = m['attacks']
m['attacks'] = [
    attack(na, MUN, 0, 'normal_attack', 'Normal Attack', base=dvec(IMP=18, PUN=18, SLA=84), cc=0.1, cd=1.5, sc=0.15,
           fire=dict(na['fire'], rate=0.917)),
    attack(sl, MUN, 1, 'slam_attack', 'Slam Attack', base=dvec(IMP=240), cc=0.1, cd=1.5, sc=0.1, range=8.0,
           falloff={'end': 8.0, 'min_factor': 0.5, 'start': 0.0}, fire=dict(sl['fire'], rate=0.917)),
    attack(hs, MUN, 2, 'heavy_slam_attack', 'Heavy Slam Attack', base=dvec(BLA=360), cc=0.1, cd=1.5, sc=0.1,
           range=9.0, falloff={'end': 9.0, 'min_factor': 0.7, 'start': 0.0}, fire=dict(hs['fire'], rate=0.917)),
    attack(hv, MUN, 3, 'heavy_attack', 'Heavy Attack', base=dvec(IMP=18, PUN=18, SLA=84), cc=0.1, cd=1.5, sc=0.15,
           fire=dict(hv['fire'], rate=0.917)),
]
b.rules['tags'].append({'id': len(b.rules['tags']), 'name': 'MACHETES_STANCE'})
b.rules['categories'].append({'bit': len(b.rules['categories']), 'name': 'Machetes'})
m.update(unique_name=MUN, name=MUN, key=1402, id=-1, image='Machete.png', mastery=1, release='8.0',
         categories=[6, len(b.rules['categories']) - 1], compat_tags=[len(b.rules['tags']) - 1],
         polarities=[], stance_polarity='zenurik', traits=[], riven_family='Machete')
m['melee'].update({
    'class': 'machete', 'family': 'Machete', 'follow_through': 0.7, 'fr_base': 0.917, 'range': 2.5, 'wind_up': 0.7,
    'stanceless': add_table('Machete', 'machete'),
    'heavy': {'attack': 3, 'kappa': 1.0, 'source': {'class_table': {'sigma': [1.2, 1.0]}}, 'hits': [
        [{'cc': [], 'mult': 1.5, 'procs': []}, {'cc': [], 'mult': 1.5, 'procs': []},
         {'cc': [], 'mult': 1.5, 'procs': ['SLA']}, {'cc': [], 'mult': 1.5, 'procs': []}],
        [{'cc': [], 'mult': 5.0, 'procs': ['KND']}]]}})
m['provenance']['sources'] = [
    {'group': 'base', 'revision': '2026-10-22T19:15:30Z', 'source': 'wiki:Module:Weapons/data/melee#Machete'},
    {'group': 'spec', 'revision': '-', 'source': '07 E-7.40 rule 2, table 9-A Machete 6× / 5× (σ_1 = 6/5 = 1.2, σ_2 = 1)'},
    {'group': 'spec', 'revision': '-', 'source': 'TV-07-10'}]
b.weapons['melee'].append(m)
b.keys.append([1402, MUN])
b.names[MUN] = 'Machete'
b.rules['melee']['heavy_class'].append({
    'breakdown': [[{'mult': 1.5, 'procs': ['SLA']}, {'mult': 1.5}, {'mult': 1.5}, {'mult': 1.5}], []],
    'class': 'machete', 'source': '07 table 9-A (Machete: 6× (150 % Bleed + 3 × 150 %) / 5×; W:Melee §Heavy Attack Damage)',
    'total': [6.0, 5.0], 'wind_up': 0.7})

# Glaive (Arcane Arachne / Power Throw on a glaive throw)
GUN = '/Lotus/Weapons/Tenno/Melee/Glaives/LightGlaive/LightGlaiveWeapon'
g = copy.deepcopy(nik)
na, sl, hs, hv = g['attacks']
thrown_fire = dict(na['fire'], rate=1.0)
g['attacks'] = [
    attack(na, GUN, 0, 'normal_attack', 'Normal Attack', base=dvec(IMP=15.75, PUN=15.75, SLA=73.5), cc=0.12, cd=2.0,
           sc=0.12, fire=dict(na['fire'], rate=1.0)),
    attack(na, GUN, 1, 'throw', 'Throw', base=dvec(IMP=17.4, PUN=17.4, SLA=81.2), cc=0.14, cd=2.1, sc=0.14,
           roles=['thrown', 'melee_projectile'], shot='thrown', range=30.0, projectile_speed=20.0, fire=thrown_fire),
    attack(na, GUN, 2, 'charged_throw', 'Charged Throw', base=dvec(IMP=34.65, PUN=34.65, SLA=161.7), cc=0.16,
           cd=2.2, sc=0.16, forced=['IMP'], punch_through=1.0, range=30.0, projectile_speed=35.0,
           roles=['charged', 'thrown', 'melee_projectile'], shot='thrown',
           fire=dict(na['fire'], rate=0.833, charge_time=1.192)),
    attack(sl, GUN, 3, 'charged_throw_recall_explosion', 'Charged Throw Recall Explosion', base=dvec(BLA=756),
           cc=0.16, cd=2.2, sc=0.16, forced=['IMP'], parent=2, radius=4.8, range=4.8,
           falloff={'end': 4.8, 'min_factor': 1.0, 'start': 0.0},
           roles=['radial', 'child', 'explosion', 'heavy', 'melee_projectile'],
           fire=dict(sl['fire'], rate=0.833, charge_time=1.192)),
    attack(sl, GUN, 4, 'slam_attack', 'Slam Attack', base=dvec(IMP=210), cc=0.12, cd=2.0, sc=0.1, range=5.0,
           falloff={'end': 5.0, 'min_factor': 0.5, 'start': 0.0}, fire=dict(sl['fire'], rate=1.0)),
    attack(hs, GUN, 5, 'heavy_slam_attack', 'Heavy Slam Attack', base=dvec(BLA=315), cc=0.12, cd=2.0, sc=0.1,
           range=6.0, falloff={'end': 6.0, 'min_factor': 0.7, 'start': 0.0}, fire=dict(hs['fire'], rate=1.0)),
]
b.rules['tags'].append({'id': len(b.rules['tags']), 'name': 'GLAIVES_STANCE'})
b.rules['categories'].append({'bit': len(b.rules['categories']), 'name': 'Glaives'})
g.update(unique_name=GUN, name=GUN, key=1403, id=-1, image='Glaive.png', mastery=1, release='7.9',
         categories=[6, 8, len(b.rules['categories']) - 1], compat_tags=[len(b.rules['tags']) - 1],
         polarities=['madurai', 'naramon'], stance_polarity='naramon', traits=[], riven_family='Glaive')
g['melee'].update({'class': 'glaive', 'family': 'Glaive', 'follow_through': 0.7, 'fr_base': 1.0, 'range': 1.25,
                   'wind_up': 1.2, 'heavy': None, 'stanceless': add_table('Glaive', 'glaive')})
g['provenance']['sources'] = [
    {'group': 'base', 'revision': '2026-10-22T19:15:30Z', 'source': 'wiki:Module:Weapons/data/melee#Glaive'},
    {'group': 'spec', 'revision': '-', 'source': '07 §9.4 (glaive throws; heavy detonation; no synthesized heavy)'},
    {'group': 'spec', 'revision': '-', 'source': '04 §3.4 (melee projectiles: second bucket)'},
    {'group': 'spec', 'revision': '-', 'source': 'bounce explosions and the uncharged recall explosion omitted from the fixture'}]
b.weapons['melee'].append(g)
b.keys.append([1403, GUN])
b.names[GUN] = 'Glaive'

# Torid (lingering child)
TUN = '/Lotus/Weapons/ClanTech/Bio/BioWeapon'
lenz = fx.weapon(b, '/Lotus/Weapons/Corpus/Bow/Longbow/CrpBow')
t = copy.deepcopy(lenz)
p0, c1, _ = t['attacks']
semi = dict(p0['fire'], trigger='semi', rate=1.5, charge_time=None)
t['attacks'] = [
    attack(p0, TUN, 0, 'grenade_impact', 'Grenade Impact', base=dvec(TOX=100), cc=0.15, cd=2.0, sc=0.23, forced=[],
           roles=['normal'], projectile_speed=40.0, fire=semi),
    attack(c1, TUN, 1, 'poison_cloud', 'Poison Cloud', base=dvec(TOX=40), cc=0.15, cd=2.0, sc=0.25, forced=[],
           parent=0, radius=3.0, range=3.0, falloff={'end': 3.0, 'min_factor': 0.0, 'start': 0.0},
           lingering={'effect_duration': 10.0, 'tick_rate': 1.0}, roles=['radial', 'child'],
           fire=dict(semi, rate=1.0)),
]
t['groups'] = [{'combinable': True, 'members': [0, 1], 'summed': True, 'volley': False}]
t.update(unique_name=TUN, name=TUN, key=1213, id=-1, image='Torid.png', mastery=4, release='8.0', **{'class': 'launcher'},
         categories=[0], compat_tags=[1, 2], exilus_polarity='naramon', polarities=[], traits=[], trigger='semi',
         riven_family='Torid', magazine={'ammo_max': 60.0, 'battery': None, 'infinite': False, 'size': 5.0},
         reload={'standard': {'time': 1.7}})
t['provenance']['sources'] = [
    {'group': 'base', 'revision': '2026-11-14T12:42:25Z', 'source': 'wiki:Module:Weapons/data/primary#Torid'},
    {'group': 'spec', 'revision': '-', 'source': '06 §9.4 E-6.25 (lingering child: EffectDuration 10 s, FireRate 1)'},
    {'group': 'spec', 'revision': '-', 'source': 'Incarnon Form attack omitted from the fixture'}]
b.weapons['primary'].append(t)
b.keys.append([1213, TUN])
b.names[TUN] = 'Torid'

# Stahlta (embedded child: EmbedDelay)
SUN = '/Lotus/Weapons/Corpus/LongGuns/CrpRubanRifle/CrpRubanRifle'
st = copy.deepcopy(lenz)
p0, c1, _ = st['attacks']
auto = dict(p0['fire'], trigger='auto', rate=6.0, charge_time=None)
alt = dict(p0['fire'], trigger='charge', rate=0.667, charge_time=1.6)
st['attacks'] = [
    attack(p0, SUN, 0, 'auto', 'Auto', base=dvec(IMP=7.28, PUN=16.64, SLA=28.08), cc=0.24, cd=1.8, sc=0.22, forced=[],
           roles=['normal'], projectile_speed=120.0, punch_through=1.2, fire=auto),
    attack(p0, SUN, 1, 'alt_fire', 'Alt-Fire', base=dvec(IMP=120, PUN=180, SLA=300), cc=0.4, cd=3.0, sc=0.32,
           forced=['IMP'], ammo_cost=20.0, roles=['alt', 'charged'], projectile_speed=300.0, fire=alt),
    attack(c1, SUN, 2, 'alt_fire_aoe', 'Alt-Fire AoE', base=dvec(RAD=1200), cc=0.4, cd=3.0, sc=0.5, forced=[],
           parent=1, radius=7.2, range=7.2, embed_delay=0.36, falloff={'end': 7.2, 'min_factor': 0.3, 'start': 0.0},
           roles=['radial', 'child', 'explosion'], fire=alt),
]
st['groups'] = [{'combinable': True, 'members': [1, 2], 'summed': True, 'volley': False}]
st.update(unique_name=SUN, name=SUN, key=1214, id=-1, image='Stahlta.png', mastery=10, release='28',
          **{'class': 'rifle'}, categories=[0, 1], compat_tags=[0, 1, 2], exilus_polarity='naramon',
          polarities=['madurai'], traits=[], trigger='auto', riven_family='Stahlta',
          magazine={'ammo_max': 300.0, 'battery': None, 'infinite': False, 'size': 40.0},
          reload={'standard': {'time': 1.4}})
st['provenance']['sources'] = [
    {'group': 'base', 'revision': '2026-11-14T12:42:25Z', 'source': 'wiki:Module:Weapons/data/primary#Stahlta'},
    {'group': 'spec', 'revision': '-', 'source': '06 §9.3 (embedded child: Alt-Fire AoE EmbedDelay 0.36 s)'},
    {'group': 'spec', 'revision': '-', 'source': '04 §2.5 D3 (radial children of a projectile: none)'}]
b.weapons['primary'].append(st)
b.keys.append([1214, SUN])
b.names[SUN] = 'Stahlta'

# ---------------------------------------------------------------------------------------------------- issue 10
tg = b.rules['toggles']
tg.append({'advanced': False, 'control': {'kind': 'optional_number', 'max': 10.0, 'min': 1.0, 'step': 1.0},
           'default': None, 'field': 'toggles.external_viral', 'group': 'target', 'help': 'toggle.t06.help',
           'id': 'T-06', 'label': 'toggle.t06.label'})
tg.append({'advanced': False, 'control': {'kind': 'number', 'max': 0.99, 'min': 0.0, 'step': 0.01},
           'default': 0.0, 'field': 'toggles.armor_strip', 'group': 'target', 'help': 'toggle.t07.help',
           'id': 'T-07', 'label': 'toggle.t07.label'})
plans = ['auto', 'light_loop', 'slide_loop', 'heavy_loop', 'build_spend', 'tennokai', 'slam_loop', 'heavy_slam_loop',
         'glaive_cycle', 'cycle_loop', 'data_attack_loop']
tg.append({'advanced': False, 'control': {'kind': 'select', 'options': [
    {'label': f'opt.{p}', 'value': p} for p in plans]}, 'default': 'auto',
    'field': 'toggles.melee_plan|toggles.melee_plans', 'group': 'melee', 'help': 'toggle.t30.help', 'id': 'T-30',
    'label': 'toggle.t30.label', 'visible_if': {'weapon_slot': ['melee']}})
tg.append({'advanced': False, 'control': {'kind': 'optional_number', 'max': 9.0, 'min': 0.0, 'step': 1.0},
           'default': None, 'field': 'toggles.aiming|toggles.zoom_level', 'group': 'scenario',
           'help': 'toggle.t38.help', 'id': 'T-38', 'label': 'toggle.t38.label'})
for d in tg:
    if d['id'] == 'T-25':
        d['field'] = 'build.variant.kitgun|build.variant.zaw|build.slots.content.item.params'
tg.sort(key=lambda d: d['id'])

# ---------------------------------------------------------------------------------------------------- issue 11
# Every record list sorted by uniqueName (pipeline.md §8.1), the modular part lists included; ChamberGrip.grip is an
# index into `grips`, so it is remapped through the uniqueNames.
old_grips = [g_['unique_name'] for g_ in b.modular['grips']]
for k in ('chambers', 'grips', 'loaders', 'strikes', 'links'):
    b.modular[k].sort(key=lambda r: r['unique_name'])
new_idx = {g_['unique_name']: n for n, g_ in enumerate(b.modular['grips'])}
for c in b.modular['chambers']:
    for bg in c['by_grip']:
        bg['grip'] = new_idx[old_grips[bg['grip']]]
    c['by_grip'].sort(key=lambda bg: bg['grip'])

fx.save(b)

# New scenario: T-06 + T-07 + an ability strip (Terrify) on an armored target.
sc = json.load(open(os.path.join(WT, 'data/fixtures/scenarios/braton_prime_lancer_l100_sp.json')))
sc['build'] = {'slots': [{'content': {'item': {'item': 3301}}, 'slot': 'gen0'},
                         {'content': {'item': {'item': 3302}}, 'slot': 'gen1'},
                         {'content': {'item': {'item': 3303}}, 'slot': 'gen2'},
                         {'content': {'item': {'item': 3304}}, 'slot': 'gen3'}],
               'variant': {}, 'weapon': 1211}
sc['scenario']['enemy'] = {'enemy': 9002, 'eximus': False, 'level': 150, 'party': 1, 'steel_path': False}
sc['scenario']['toggles'] = {'armor_strip': 0.5, 'external_viral': 10}
sc['scenario']['buffs'] = [{'buff': 3615}, {'buff': 3614, 'caster': {'ally': {'duration': 1.0, 'strength': 2.0}}}]
fx.wr('../scenarios/gorgon_heavy_gunner_l150_strip_viral.json', sc)

for sec, row in SOURCES:
    print(row)

# ---------------------------------------------------------------------------------------------------- issue 14
# Revision timestamps of the pinned wiki revisions (research/tools/wiki-pins.tsv: orokin revid 2816758,
# themurmur revid 2731757), written like the other wiki sources of the manifest.
man = fx.rd('manifest.json')
PIN = {'Module:Enemies/data/orokin': '2026-10-08T01:54:49Z', 'Module:Enemies/data/themurmur': '2026-02-24T22:10:01Z'}
for s in man['sources']:
    if s['id'] in PIN:
        s['revision'] = s['timestamp'] = PIN[s['id']]
fx.wr('manifest.json', man)

# Rubico scenario: the zoom level explicit (T-38 zoom_level 1 = 6x, the level all_max picks anyway, D-406).
rp = os.path.join(WT, 'data/fixtures/scenarios/rubico_eximus_heavy_gunner_l58_overguard.json')
r = json.load(open(rp))
r['scenario']['toggles']['zoom_level'] = 1
fx.wr('../scenarios/rubico_eximus_heavy_gunner_l58_overguard.json', r)
