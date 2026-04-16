//! ignis0 — stage-0 IL interpreter for IgnisSynth.
//!
//! This crate implements the contract in
//! `../../kernel/IGNITION-BOOTSTRAP.md` against the IL specified
//! in `../../kernel/IL.md`.
//!
//! It is the external ignition substrate named by axiom A9. It
//! is not part of IgnisSynth; it is the software IgnisSynth runs
//! on top of, analogous to a CPU for an ordinary program.

pub mod capability;
pub mod exec;
pub mod fixed_point;
pub mod opcode;
pub mod parser;
pub mod pretty;
pub mod registry;
pub mod store;
pub mod value;
pub mod wire;

pub use capability::{
    builtin_cap_id, CapabilityInvoker, CapabilityRegistry, GpuComputeConfig, InferenceConfig,
    GPU_COMPUTE_CAP_DESCRIPTOR, INFER_CAP_DESCRIPTOR,
};
pub use exec::{ExecState, ExecVerdict, Frame, Interpreter};
pub use fixed_point::{FixedPointCheck, FixedPointVerdict};
pub use opcode::Opcode;
pub use parser::{parse_form_lines, ParseError};
pub use pretty::{opcode_to_line, pretty_print, pretty_print_with_header};
pub use registry::{FormRegistry, LoadedForm};
pub use store::SubstanceStore;
pub use value::{Hash, SubstanceHash, TrapKind, Value};
pub use wire::{decode_form, encode_form, Form, WireError};
