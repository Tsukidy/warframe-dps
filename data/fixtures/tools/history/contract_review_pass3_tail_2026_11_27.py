"""Contract review fixes, pass 3 tail (2026-11-27): the fixture edit of CONTRACTS.md "Contract review fixes (pass 3
tail)".

Historical record (README.md "Tools"); not idempotent. Run once on the fixture of ccr-659c593d-av1db8 at cc80c6e:

    python3 -I data/fixtures/tools/history/contract_review_pass3_tail_2026_11_27.py

Item ids are those of docs/project/inputs/contract-review-pass3-tail.json.

- P3-3 Mandonel field (`wfd:buff/mandonel_field`, key 3616): a weapon-made buffing field, so its Radiation line carries
  `dot_scaling = false` (03 §9.3 excludes weapon-made fields from E^dot; 04 §5.6; effect.rs `Effect.dot_scaling`), the
  DC-11 rule of Electric Shield. No numeric effect today (Radiation has no DoT).
- manifest.json: `data_version` fixture-mini.5.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..'))
import fixture_io as fx  # noqa: E402

b = fx.load()

# ---- P3-3 Mandonel field -----------------------------------------------------------------------------------------------
mf = fx.item(b, 'wfd:buff/mandonel_field')
els = [e for e in mf['effects'] if 'element' in e]
assert [e['element'] for e in els] == ['RAD'], els
rad = els[0]
assert 'dot_scaling' not in rad, 'already applied'
rad['dot_scaling'] = False
rad['note'] = ('Radiation "+100% (multiplicative)": form UNVERIFIED, Decision E[RAD] +1.00 (04 §5.6, OQ-415); '
               'no DoT scaling from a weapon-made buffing field (03 §9.3; contract review P3-3)')
mf['sources'] = mf['sources'] + ['03 §9.3 (weapon-made fields outside E^dot)']

fx.save(b)

man = fx.rd('manifest.json')
assert man['data_version'] == 'fixture-mini.4', man['data_version']
man['data_version'] = 'fixture-mini.5'
fx.wr('manifest.json', man)
