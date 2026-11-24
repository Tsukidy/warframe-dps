//! Messages and message keys (server-api.md §2): owned by T01b (work-breakdown.md §8, T01b), which adds `Message`,
//! `Severity` and the per-area key enums (`message/{api,doc,engine,opt,data,import,legality}.rs`). T01a only declares
//! this module. The key and argument types that domain types already carry are [`crate::i18n::MessageKey`] and
//! [`crate::i18n::MsgArg`] (T01a); T01b re-exports them here (CONTRACTS.md, T01a handover).
