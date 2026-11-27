//! The passes of 08 §6.4 (V views, L layered, L0 layered with armor ignored for parity) and their parallel run loop:
//! chunks of runs on the rayon pool with feature `parallel`, results written per run and reduced in run order (I-5),
//! cancellation checked between runs. Owner: T16.
