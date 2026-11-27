//! Capacity, polarity and Forma (optimizer.md §5.2; spec 09 §6, E-9.14-E-9.19, D-092).
//!
//! Owner: T32. The capacity mode is `constraints.capacity.mode` (`ignore`, `current`, `any_forma` (default),
//! `budget(N)`), never the scenario's T-35 toggle; the allowed Forma types are `CapacityRule::effective` (server-api.md
//! §8.1, CONTRACTS.md T01d). Drain is a knapsack constraint pruned with the suffix minimum-drain table (drain after the
//! best possible polarity, a valid lower bound); at a leaf, feasibility and the minimum Forma count come from
//! `wfd-data::legality::min_forma` (09 E-9.16), the function the calculator's capacity report uses.

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "T32: every kept build is capacity-feasible under the job's mode (spec 09 E-9.15)"]
    fn capacity_kept_builds_are_feasible() {
        todo!("T32")
    }
}
