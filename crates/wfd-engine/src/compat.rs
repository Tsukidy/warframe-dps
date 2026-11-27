//! Compatibility-profile helpers (engine.md §10; spec 00 §4.6, §5). `Profile::Compat(CompatFlags)` switches register
//! rows whose switch kind is `Compat` to the inspiration's behaviour, for oracle tests only; each branch is a one-line
//! flag test in the stage that owns the row (`if ctx.compat.contains(CompatFlags::D001_QUANTIZATION_GRID)`, with a
//! `// D-0xx` comment). The active flags come from `wfd_model::Profile::flags` into `CompiledScenario.profile` and
//! [`crate::StageCtx::compat`]. Every stage that runs a register row's code path records the row through
//! [`crate::TraceSink::touch_row`], flag or not (`touched_rows`, 08 §11.4). Owner: T17.
