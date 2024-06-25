//! Fuzzing harness for SVM programs.
//!
//! Note, although the fuzz harness provides an easy way to fuzz programs with
//! Mollusk, it is not required. Developers can use this fuzz harness on their
//! own custom SVM entrypoint. Hence the distinction between fixture types and
//! Mollusk types.

pub mod fixture;
