//! ignis0 CLI — a thin wrapper around the library.
//!
//! Usage:
//!   ignis0 fixed-point          Run the A9.3 fixed-point check.
//!   ignis0 pretty-print <file>  Parse a .form source file and
//!                               pretty-print the opcode sequence.
//!   ignis0 caps                 Print canonical CapIds for built-in capabilities.
//!   ignis0 version              Print the scaffold version.
//!   ignis0 help                 Print this message.

use ignis0::capability::{
    builtin_cap_id, GPU_COMPUTE_CAP_DESCRIPTOR, INFER_CAP_DESCRIPTOR,
};
use ignis0::fixed_point::{FixedPointCheck, FixedPointVerdict};
use ignis0::parser::parse_form_lines;
use ignis0::pretty::pretty_print;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");

    match cmd {
        "fixed-point" => run_fixed_point(),
        "pretty-print" => run_pretty_print(&args),
        "caps" => run_caps(),
        "version" => {
            println!("ignis0 v{}", env!("CARGO_PKG_VERSION"));
        }
        "help" | _ => print_help(),
    }
}

fn print_help() {
    println!(
        "ignis0 — stage-0 IL interpreter scaffold\n\
         \n\
         Usage:\n\
         \n\
             ignis0 fixed-point          Run the A9.3 fixed-point check\n\
             ignis0 pretty-print <file>  Parse and pretty-print a .form file\n\
             ignis0 caps                 Print canonical CapIds for built-in capabilities\n\
             ignis0 version              Print the scaffold version\n\
             ignis0 help                 Print this message\n\
         \n\
         Capability features (rebuild with --features to enable):\n\
             --features infer   HTTP inference backend (ollama / llama.cpp / vllm)\n\
             --features gpu     wgpu GPU compute shader dispatch\n\
             --features compute both infer + gpu\n\
         \n\
         See ../kernel/IGNITION-BOOTSTRAP.md for the full contract."
    );
}

/// Print the canonical CapIds for built-in capabilities.
///
/// These hashes are stable; they are the values that IL code must push
/// before INVOKE n to reach a built-in capability backend. The hash is
/// derived from the descriptor string, so it cannot change without
/// bumping the descriptor and all Forms that reference the old hash.
fn run_caps() {
    let infer_id = builtin_cap_id(INFER_CAP_DESCRIPTOR);
    let gpu_id = builtin_cap_id(GPU_COMPUTE_CAP_DESCRIPTOR);

    println!("ignis0 built-in capability IDs");
    println!();
    println!(
        "  Synthesis/infer/v1  descriptor = {:?}",
        std::str::from_utf8(INFER_CAP_DESCRIPTOR).unwrap_or("(non-utf8)")
    );
    println!("                      cap_id     = {}", infer_id.short());
    println!();
    println!(
        "  Compute/gpu/v1      descriptor = {:?}",
        std::str::from_utf8(GPU_COMPUTE_CAP_DESCRIPTOR).unwrap_or("(non-utf8)")
    );
    println!("                      cap_id     = {}", gpu_id.short());
    println!();
    println!("IL usage (inference, INVOKE 2):");
    println!("  PUSH <prompt_hash>   ; Hash of a Bytes/v1 substance");
    println!("  PUSH <params_hash>   ; Hash of InferParams/v1 or BOTTOM");
    println!("  PUSH {}  ; infer_cap_id (above)", infer_id.short());
    println!("  INVOKE 2");
    println!();
    println!("IL usage (GPU compute, INVOKE 3):");
    println!("  PUSH <shader_hash>   ; Hash of a Bytes/v1 substance (WGSL source)");
    println!("  PUSH <input_hash>    ; Hash of a Bytes/v1 substance (input bytes)");
    println!("  PUSH <output_size>   ; Nat — output buffer size in bytes");
    println!("  PUSH {}  ; gpu_cap_id (above)", gpu_id.short());
    println!("  INVOKE 3");
}

fn run_pretty_print(args: &[String]) {
    let path = match args.get(2) {
        Some(p) => p,
        None => {
            eprintln!("Usage: ignis0 pretty-print <file>");
            eprintln!("       <file> must be a line-oriented scaffold .form source.");
            std::process::exit(1);
        }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            std::process::exit(1);
        }
    };

    match parse_form_lines(&source) {
        Ok(code) => {
            println!("; pretty-printed from: {}", path);
            println!("; {} opcode(s)", code.len());
            println!();
            print!("{}", pretty_print(&code));
        }
        Err(e) => {
            eprintln!("parse error in {}: {}", path, e);
            std::process::exit(1);
        }
    }
}

fn run_fixed_point() {
    let mut check = FixedPointCheck::new();
    let verdict = check.run();

    match verdict {
        FixedPointVerdict::Pass {
            direct,
            indirect_1,
            indirect_2,
            indirect_1_max_depth,
            indirect_2_max_depth,
        } => {
            println!("fixed-point: PASS");
            println!("  direct     = {:?}", direct);
            println!("  indirect_1 = {:?} (max frame depth {})", indirect_1, indirect_1_max_depth);
            println!("  indirect_2 = {:?} (max frame depth {})", indirect_2, indirect_2_max_depth);
            println!("\nA9.3 necessary condition holds. ignis0 is faithful to");
            println!("the IL on the canonical case across all three levels.");
        }
        FixedPointVerdict::Incomplete {
            direct,
            indirect_1_status,
            indirect_2_status,
        } => {
            println!("fixed-point: INCOMPLETE (scaffold)");
            println!("  direct            = {:?}", direct);
            println!("  indirect_1 status = {}", indirect_1_status);
            println!("  indirect_2 status = {}", indirect_2_status);
            println!("\nThe direct case passed. The indirect cases are stubbed in this");
            println!("scaffold and will be exercised once the IL parser and CALL opcode");
            println!("are wired up. A9.3 requires all three levels to agree before");
            println!("ignition may proceed; this scaffold is not yet ignition-ready.");
            std::process::exit(2);
        }
        FixedPointVerdict::DirectFailed(msg) => {
            eprintln!("fixed-point: FAIL (direct case)");
            eprintln!("  {}", msg);
            eprintln!("\nThis is an ignis0 bug. A correct stage-0 must pass the direct case.");
            std::process::exit(1);
        }
        FixedPointVerdict::Disagreed { direct, indirect, level } => {
            eprintln!("fixed-point: FAIL (level {} disagrees with direct)", level);
            eprintln!("  direct   = {:?}", direct);
            eprintln!("  indirect = {:?}", indirect);
            eprintln!("\nignis0 is not faithful to the IL on the canonical case.");
            eprintln!("Ignition must halt per A9.3.");
            std::process::exit(1);
        }
    }
}
