//! ignis0 — stage-0 IL interpreter for IgnisSynth.
//!
//! This crate implements the contract in
//! `../../kernel/IGNITION-BOOTSTRAP.md` against the IL specified
//! in `../../kernel/IL.md`.
//!
//! It is the external ignition substrate named by axiom A9. It
//! is not part of IgnisSynth; it is the software IgnisSynth runs
//! on top of, analogous to a CPU for an ordinary program.

pub mod exec;
pub mod fixed_point;
pub mod opcode;
pub mod parser;
pub mod registry;
pub mod store;
pub mod value;

pub use exec::{ExecState, ExecVerdict, Frame, Interpreter};
pub use fixed_point::{FixedPointCheck, FixedPointVerdict};
pub use opcode::Opcode;
pub use parser::{parse_form_lines, ParseError};
pub use registry::{FormRegistry, LoadedForm};
pub use store::SubstanceStore;
pub use value::{Hash, TrapKind, Value};
