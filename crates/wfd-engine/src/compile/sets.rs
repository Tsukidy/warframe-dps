//! Set bonuses (09 E-9.40 counting, active weapon rule; `Per::SetPiece` finalizer at L2). The count is
//! `wfd_model::sets::set_counts(bundle, build)`, the one implementation of 09 E-9.40 that `wfd-data`'s
//! `legality::set_counts` also delegates to and that TV-09-09 checks (AD-24; contract review API-6): this module
//! never counts pieces itself. Owner: T06.
