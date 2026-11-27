"""Contract review fixes, pass 3 (2026-11-27): the fixture edits of CONTRACTS.md "Contract review fixes (pass 3)".

Historical record (README.md "Tools"); not idempotent. Run once on the fixture of ccr-659c593d-av1db8 at 2929f52:

    python3 -I data/fixtures/tools/history/contract_review_pass3_2026_11_27.py

Item ids are those of docs/project/inputs/contract-review-recheck.json.

- DC-11 Electric Shield: its Electricity line carries `dot_scaling = false` (04 §5.6, 03 §9.3; domain-model.md §16.13).
- DC-12 custom p.k `status_chance_mult`: a target-side factor of M_sc,t (02 E-2.11, 04 §10), so the effect is
  `mult = "custom_sc_vuln"` (stage `sc_vuln`; the group copied from data/curated/rules/mult_groups.toml into
  rules.json); the weapon-side `sc_mul` (M_sc,w) keeps a user in the new buff `wfd:buff/mandonel_field` (04 §5.6
  "Weapon-made buffing field: Mandonel charged shot", key 3616).
- ENG-R7 / NEW-CR-8 Prelude of Might (Furis): CDM0c and θ_P as the registry rows `cd_base_cond` 3.0 and
  `cd_base_cond_thr` 0.40 (02 E-2.5 and its CDM0c table; 04 §5.12 "cd_base_cond, not cd_base_add"); the unconditional
  CDM0+ row `cd_base_add` keeps a user in the new engine-only perk `wfd:perk/braton/critical_parallel` (Braton
  Incarnon EVO4 "Critical Parallel", family 4 index 0, key 3516).
- N-8 abilityCombo: the custom stat's parameter follows T-11 (0 = off, 1 … 12, step 1; 00 §6.1, scenario.rs
  `ScenarioDoc.custom` conversion).
- manifest.json: `data_version` fixture-mini.4.
"""
import json
import os
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..'))
import fixture_io as fx  # noqa: E402

RULES = os.path.join(HERE, '..', '..', '..', 'curated', 'rules')
BRATON = 'W:Braton_Incarnon_Genesis L107-115 (rev 2824593)'


def toml(name):
    with open(os.path.join(RULES, name), 'rb') as f:
        return tomllib.load(f)


b = fx.load()

# ---- DC-11 Electric Shield ---------------------------------------------------------------------------------------------
es = fx.item(b, 'wfd:buff/electric_shield')
ele = es['effects'][0]
assert ele.get('element') == 'ELE' and 'dot_scaling' not in ele, 'already applied'
ele['dot_scaling'] = False
ele['note'] = 'no DoT scaling from a buffing field (04 §5.6, 03 §9.3; contract review DC-11)'
es['sources'] = ['W:Electric_Shield', '04 catalogue Electric Shield', '04 §5.6 (no Electricity DoT scaling)',
                 '03 §9.3', 'domain-model.md §16.13']

# ---- DC-12 custom p.k on the target side; Mandonel field (M_sc,w) ------------------------------------------------------
real_groups = {g['id']: g for g in toml('mult_groups.toml')['group']}
g = dict({'twice_on_two_bucket': False}, **real_groups['custom_sc_vuln'])
assert g['stage'] == 'sc_vuln' and g['dot'] == 'none'
assert not any(x['id'] == 'custom_sc_vuln' for x in b.rules['mult_groups']), 'already applied'
b.rules['mult_groups'].append(g)

pk = fx.item(b, 'wfd:custom/status_chance_mult')
assert pk['effects'] == [{'stat': 'sc_mul', 'value': {'param': 'value'}}], pk['effects']
pk['effects'] = [{'mult': 'custom_sc_vuln', 'value': {'param': 'value'}}]
pk['sources'] = ['04 §10 (status_chance_mult: target-side factor (1 + x) in sc_vuln)', '00 §6.3',
                 '02 E-2.11 (M_sc,t, a separate factor)']

shield = fx.item(b, 'wfd:buff/electric_shield')
mf = json.loads(json.dumps(shield))
mf.update({
    'effects': [
        {'note': 'P_t +1.00 on IPS (04 §5.6)', 'stat': f'phys.{t}', 'value': 1.0} for t in ('IMP', 'PUN', 'SLA')
    ] + [
        {'element': 'RAD', 'note': 'Radiation "+100% (multiplicative)": form UNVERIFIED, Decision E[RAD] +1.00 '
                                   '(04 §5.6, OQ-415)', 'order': 'non_combining', 'value': 1.0},
        {'stat': 'cc_abs', 'value': 0.10},
        {'note': 'cd_field_mult ×1.5 per field = 1 + 0.5', 'stat': 'cd_field_mul', 'value': 0.5},
        {'note': 'sc_mult_w ×2 = 1 + 1: M_sc,w, weapon side (02 E-2.11)', 'stat': 'sc_mul', 'value': 1.0},
    ],
    'key': 3616,
    'kind': {'buff': {'ability_source': False, 'group': 'squad', 'inspiration_key': None,
                      'strength_scaled': False, 'subsumable': False, 'toggle': None}},
    'name': 'wfd:buff/mandonel_field',
    'sources': ['04 §5.6 (Weapon-made buffing field: Mandonel charged shot; one field source per shot)',
                'W:Buffing_Fields §Sources', '02 E-2.11 (sc_mult_w)', '02 §4.3 (M_cdf ×1.5 per field)',
                '02 §4.2 (ΣCC_abs +0.10)', 'EC-04-94 (user-entered class (u) field buff)'],
    'status': 'verified',
    'unique_name': 'wfd:buff/mandonel_field',
    'upstream_text': None,
})
b.items['buff'].append(mf)
b.names['wfd:buff/mandonel_field'] = 'Mandonel field'
b.keys.append([3616, 'wfd:buff/mandonel_field'])

# ---- ENG-R7 / NEW-CR-8 Prelude of Might; Braton Critical Parallel (CDM0+) ---------------------------------------------
pm = fx.item(b, 'wfd:perk/furis/prelude_of_might')
assert pm['effects'][0]['stat'] == 'cd_base_add', 'already applied'
pm['effects'] = [
    {'note': 'CDM0c of 02 E-2.5, the last f32 addend of x32 while the final CC is below θ_P', 'stat': 'cd_base_cond',
     'value': 3.0},
    {'note': 'θ_P (strict: CC_i < 0.40)', 'stat': 'cd_base_cond_thr', 'value': 0.40},
]
pm['sources'] = ['W:Furis_Incarnon_Genesis', 'F:mods §9', '02 E-2.5 (CDM0c table: Furis θ_P 0.40, X 3.0)',
                 '04 §5.12 (cd_base_cond, not cd_base_add)', 'domain-model.md §16.10']

se = fx.item(b, 'wfd:perk/braton/survivors_edge')
cp = json.loads(json.dumps(se))
cp.update({
    'effects': [
        {'stat': 'cc_base_add',
         'value': {'variants': {'braton': 0.16, 'braton_prime': 0.18, 'braton_vandal': 0.16, 'mk1_braton': 0.18}}},
        {'stat': 'cd_base_add',
         'value': {'variants': {'braton': 0.4, 'braton_prime': 0.2, 'braton_vandal': 0.4, 'mk1_braton': 0.5}}},
    ],
    'key': 3516,
    'kind': {'incarnon_perk': {'family': 4, 'index': 0, 'tier': 4}},
    'name': 'wfd:perk/braton/critical_parallel',
    'sources': [BRATON, '02 §4.2 CC0+ (Incarnon base crit chance)',
                '02 §4.3 CDM0+ ("Increase Base Critical Damage Multiplier", E-2.5)'],
    'unique_name': 'wfd:perk/braton/critical_parallel',
    'upstream_text': 'Increase Base Critical Chance by +X. Increase Base Critical Damage Multiplier by +Y.',
})
b.items['incarnon'].append(cp)
b.names['wfd:perk/braton/critical_parallel'] = 'Critical Parallel'
b.keys.append([3516, 'wfd:perk/braton/critical_parallel'])

# ---- N-8 abilityCombo ---------------------------------------------------------------------------------------------------
ac = fx.item(b, 'wfd:custom/abilitycombo')
assert ac['params'][0]['max'] == 1000.0, 'already applied'
ac['params'][0].update({'default': 0.0, 'max': 12.0, 'min': 0.0, 'step': 1.0})
ac['sources'] = ['04 §10', '00 §6.3', '00 §6.1 T-11 (1 … 12, off)',
                 'domain-model.md §9.3 T-26 (a T-11 alias: resolve moves it into toggles.ability_combo; M-18; '
                 'conversion N-8: 0 = off, round and clamp to 1 … 12)']

fx.save(b)

man = fx.rd('manifest.json')
man['data_version'] = 'fixture-mini.4'
fx.wr('manifest.json', man)
