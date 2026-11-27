//! Mechanic ids (08 E-8.16; engine.md §8.2): `MechanicId` is the FNV-1a 32-bit hash of the mechanic id **string**,
//! computed at L0. The fixed ids of 02 4.8, 07 and T-41 are the const list [`FIXED_MECHANIC_IDS`]; derived ids
//! (`"xh/<source>/<name>"`, `"dup/<name>"`, `"xd/<name>"`, `"eff/<effect key>"`, `"eff/<effect id>/phase"`) are formed
//! from the bundle at L0. There is no Rust enum of mechanic ids. Owner: T16.

use crate::error::EngineError;

/// FNV-1a offset basis (08 E-8.16).
pub const FNV_OFFSET: u32 = 0x811C_9DC5;
/// FNV-1a prime (08 E-8.16).
pub const FNV_PRIME: u32 = 0x0100_0193;

/// The fixed mechanic id strings, in the draw order of 08 E-8.17 (02 4.8, 02 E-2.17, E-2.17b, E-2.20; 07; T-41).
pub const FIXED_MECHANIC_IDS: &[&str] = &[
    "hit_frac",
    "ms_pellets",
    "crit_roll",
    "crit_vig",
    "sc_count",
    "sc_type",
    "cedo_type",
    "aug_type",
    "aug_ele",
    "ib",
    "welt",
    "hm",
    "st",
    "enc",
    "enc_type",
    "deb",
    "deb_type",
    "deb_comp",
    "combo_acc",
    "combo_rc",
    "combo_curse",
    "glaive_volatile",
];

/// A mechanic id: FNV-1a 32-bit of the id string (counter word c1 of E-8.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MechanicId(pub u32);

/// 08 E-8.16: `rng_mech(name)` = FNV-1a 32-bit of the UTF-8 bytes of `name`. T16.
pub fn rng_mech(name: &str) -> u32 {
    let _ = name;
    todo!("T16")
}

/// Every mechanic id a scenario can use (`CompiledScenario.mech_ids`), checked distinct at L0.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MechIdTable {
    ids: Vec<(MechanicId, Box<str>)>,
}

impl MechIdTable {
    /// An empty table.
    pub fn new() -> Self {
        MechIdTable::default()
    }

    /// Registers an id string (from [`crate::mechanics::MechanicMc::mechanic_ids`] or the fixed list) and returns its
    /// hash; registering the same string again returns the same id. A second, different string with the same hash is
    /// `EngineError::MechanicIdCollision { a, b }` naming both strings (engine.md §8.2, 08 E-8.17). T16.
    pub fn insert(&mut self, name: &str) -> Result<MechanicId, EngineError> {
        let _ = name;
        todo!("T16")
    }

    /// The id of a registered string.
    pub fn get(&self, name: &str) -> Option<MechanicId> {
        self.ids
            .iter()
            .find(|(_, n)| &**n == name)
            .map(|(id, _)| *id)
    }

    /// Number of registered ids.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether no id is registered.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_ids_are_unique_strings() {
        let set: std::collections::BTreeSet<_> = FIXED_MECHANIC_IDS.iter().collect();
        assert_eq!(set.len(), FIXED_MECHANIC_IDS.len());
        assert_eq!(FIXED_MECHANIC_IDS.len(), 22, "TV-08-02 lists 22 ids");
    }

    #[test]
    #[ignore = "T16"]
    fn tv_08_02_fnv_ids() {
        assert_eq!(rng_mech(""), 0x811C9DC5);
        assert_eq!(rng_mech("a"), 0xE40C292C);
        assert_eq!(rng_mech("foobar"), 0xBF9CF968);
        let expect = [
            ("ms_pellets", 0xf351e291),
            ("crit_roll", 0xa4bc7899),
            ("crit_vig", 0x2f2fe9e4),
            ("sc_count", 0xe545931b),
            ("sc_type", 0x95eb190c),
            ("cedo_type", 0xe304b3a9),
            ("aug_type", 0xf859fe59),
            ("aug_ele", 0x234f47cf),
            ("ib", 0x3d387452),
            ("welt", 0xecb57171),
            ("hm", 0x643af04e),
            ("st", 0x4b522242),
            ("enc", 0x658e6dcb),
            ("enc_type", 0x6e512494),
            ("deb", 0xc1597840),
            ("deb_type", 0xa6c40bc5),
            ("deb_comp", 0xc8e04db2),
            ("combo_acc", 0x4feff83f),
            ("combo_rc", 0xa570f7a1),
            ("combo_curse", 0x151631cc),
            ("glaive_volatile", 0xd5e66d86),
            ("hit_frac", 0x34e8d3d7),
        ];
        for (name, h) in expect {
            assert_eq!(rng_mech(name), h, "{name}");
        }
        let mut table = MechIdTable::new();
        for name in FIXED_MECHANIC_IDS {
            table
                .insert(name)
                .expect("fixed ids are distinct (TV-08-02)");
        }
        assert_eq!(table.len(), FIXED_MECHANIC_IDS.len());
    }

    #[test]
    #[ignore = "T16"]
    fn colliding_strings_are_an_error_naming_both() {
        // FNV-1a 32 collides on "costarring" and "liquid" (both 0x5e4daa9d): the check needs the strings.
        assert_eq!(rng_mech("costarring"), rng_mech("liquid"));
        let mut table = MechIdTable::new();
        let id = table.insert("costarring").expect("first string");
        assert_eq!(table.insert("costarring").expect("same string again"), id);
        match table.insert("liquid") {
            Err(EngineError::MechanicIdCollision { a, b }) => {
                assert_eq!((&*a, &*b), ("costarring", "liquid"));
            }
            other => panic!("expected a collision error, got {other:?}"),
        }
    }
}
