//! ignis0 CLI — a thin wrapper around the library.
//!
//! Usage:
//!   ignis0 fixed-point          Run the A9.3 fixed-point check.
//!   ignis0 pretty-print <file>  Parse a .form source file and
//!                               pretty-print the opcode sequence.
//!   ignis0 caps                 Print canonical CapIds for built-in capabilities.
//!   ignis0 verify <file>        Run the envelope verifier; exit 0 if admitted.
//!   ignis0 run <file>           Verify, then execute under the gated mode.
//!   ignis0 explain <file>       Print a structured explanation of verify/run.
//!   ignis0 derive <file> --rule <id> [--out <path>]
//!                               Derive a child envelope from an existing one.
//!   ignis0 version              Print the scaffold version.
//!   ignis0 help                 Print this message.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ignis0::capability::{
    builtin_cap_id, GPU_COMPUTE_CAP_DESCRIPTOR, INFER_CAP_DESCRIPTOR,
};
use ignis0::derive::derive_form;
use ignis0::envelope::FormEnvelope;
use ignis0::fixed_point::{FixedPointCheck, FixedPointVerdict};
use ignis0::parser::parse_form_lines;
use ignis0::pretty::pretty_print;
use ignis0::runner::{run_envelope, EnvelopeMode, OpDecision};
use ignis0::verify::{verify, Ledger, VerifyError};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");

    match cmd {
        "fixed-point" => {
            run_fixed_point();
            ExitCode::SUCCESS
        }
        "pretty-print" => {
            run_pretty_print(&args);
            ExitCode::SUCCESS
        }
        "caps" => {
            run_caps();
            ExitCode::SUCCESS
        }
        "verify" => run_verify_cmd(&args),
        "run" => run_run_cmd(&args),
        "explain" => run_explain_cmd(&args),
        "derive" => run_derive_cmd(&args),
        "version" => {
            println!("ignis0 v{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            print_help();
            ExitCode::SUCCESS
        }
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
         \n\
         Envelope (derivation-gated execution) commands:\n\
         \n\
             ignis0 verify  <file> [--ledger <dir>]\n\
                                         Run verifier; exit 0 if admitted, 1 if refused\n\
             ignis0 run     <file> [--ledger <dir>]\n\
                                         Verify, then execute under the gated mode\n\
             ignis0 explain <file> [--ledger <dir>]\n\
                                         Print a structured reason for verify/run\n\
             ignis0 derive  <file> --rule <id> [--out <path>]\n\
                                         Derive a child envelope from an existing one\n\
         \n\
         Other:\n\
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
            println!("\nThe direct case passed but one or both indirect levels did not");
            println!("produce a comparable result. Post-v0.2.3 this path should not fire;");
            println!("if it does, treat it as an ignis0 bug. A9.3 requires all three");
            println!("levels to agree before ignition may proceed.");
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

// ── Envelope commands ────────────────────────────────────────────────

/// Pull the positional file argument and the optional `--ledger <dir>`
/// flag out of an argv slice that looks like:
///     ignis0 <subcommand> <file> [--ledger <dir>]
/// Returns the file path and the resolved ledger directory.
fn parse_file_and_ledger(args: &[String], subcommand: &str) -> Option<(PathBuf, PathBuf)> {
    let path = args.get(2)?.clone();
    let mut ledger_dir: Option<PathBuf> = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--ledger" => {
                let dir = args.get(i + 1).cloned()?;
                ledger_dir = Some(PathBuf::from(dir));
                i += 2;
            }
            other => {
                eprintln!("ignis0 {}: unrecognised argument {:?}", subcommand, other);
                return None;
            }
        }
    }

    let path = PathBuf::from(path);
    // Default ledger: the directory containing the input file. This
    // makes single-directory demos work without explicit flags.
    let resolved_ledger = match ledger_dir {
        Some(d) => d,
        None => path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    Some((path, resolved_ledger))
}

fn load_envelope_and_ledger(
    path: &Path,
    ledger_dir: &Path,
) -> Result<(FormEnvelope, Ledger), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let env = FormEnvelope::from_json_bytes(&bytes)
        .map_err(|e| format!("cannot parse {}: {}", path.display(), e))?;
    let mut ledger = Ledger::load_from_dir(ledger_dir)
        .map_err(|e| format!("cannot load ledger from {}: {}", ledger_dir.display(), e))?;
    // Make sure the envelope under inspection is itself in the ledger
    // so that *its* descendants (if any are loaded later) can resolve
    // it as a parent. This is idempotent.
    ledger.insert(env.clone());
    Ok((env, ledger))
}

fn run_verify_cmd(args: &[String]) -> ExitCode {
    let Some((path, ledger_dir)) = parse_file_and_ledger(args, "verify") else {
        eprintln!("Usage: ignis0 verify <file> [--ledger <dir>]");
        return ExitCode::from(2);
    };
    let (env, ledger) = match load_envelope_and_ledger(&path, &ledger_dir) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("verify: {}", e);
            return ExitCode::from(2);
        }
    };
    match verify(&env, &ledger) {
        Ok(out) => {
            println!("verify: ADMIT  ({})", out.proof_status_label());
            for w in &out.warnings {
                println!("  warning: {}", w);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("verify: REFUSE — {}", e);
            ExitCode::from(1)
        }
    }
}

fn run_run_cmd(args: &[String]) -> ExitCode {
    let Some((path, ledger_dir)) = parse_file_and_ledger(args, "run") else {
        eprintln!("Usage: ignis0 run <file> [--ledger <dir>]");
        return ExitCode::from(2);
    };
    let (env, ledger) = match load_envelope_and_ledger(&path, &ledger_dir) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("run: {}", e);
            return ExitCode::from(2);
        }
    };
    let outcome = match verify(&env, &ledger) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("run: REFUSED at gate — {}", e);
            return ExitCode::from(1);
        }
    };
    let result = run_envelope(&env, &outcome);
    println!("run: mode={}", result.mode.label());
    for (i, decision) in result.decisions.iter().enumerate() {
        match decision {
            OpDecision::Executed { effect } => println!("  [{}] EXEC      {}", i, effect),
            OpDecision::SkippedRestricted { op_name } => {
                println!("  [{}] SKIP-R    {} (restricted mode: side-effect denied)", i, op_name)
            }
            OpDecision::Denied => println!("  [{}] DENIED   (mode == denied)", i),
        }
    }
    println!(
        "run: {} executed, {} skipped (restricted), {} denied",
        result.executed_count(),
        result.skipped_count(),
        result.denied_count()
    );
    if matches!(result.mode, EnvelopeMode::Denied) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_explain_cmd(args: &[String]) -> ExitCode {
    let Some((path, ledger_dir)) = parse_file_and_ledger(args, "explain") else {
        eprintln!("Usage: ignis0 explain <file> [--ledger <dir>]");
        return ExitCode::from(2);
    };
    let (env, ledger) = match load_envelope_and_ledger(&path, &ledger_dir) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("explain: {}", e);
            return ExitCode::from(2);
        }
    };
    println!("explain: {}", path.display());
    println!("  form_id        = {}", env.form_id);
    println!("  hash (claim)   = {}", short_hash(&env.hash));
    let canonical = env.compute_canonical_hash();
    println!("  hash (canonical) = {}", short_hash(&canonical));
    println!("  parents        = {:?}", env.parents);
    println!("  rule           = {}", env.rule);
    println!("  proof_status   = {:?}", env.proof_status);
    println!("  open_obligations = {:?}", env.open_obligations);
    println!("  capabilities   = {:?}", env.capabilities);
    println!("  payload ops    = {}", env.payload.ops.len());
    println!();
    match verify(&env, &ledger) {
        Ok(out) => {
            println!("verdict: ADMIT under proof_status={:?}", out.proof_status);
            if !out.warnings.is_empty() {
                println!("warnings:");
                for w in &out.warnings {
                    println!("  - {}", explain_one(w));
                }
            }
            // Also print what the runner would do.
            let mode = EnvelopeMode::for_status(out.proof_status);
            println!("runner mode (chosen from proof_status): {}", mode.label());
            if matches!(mode, EnvelopeMode::Restricted) {
                let restricted_ops: Vec<_> = env
                    .payload
                    .ops
                    .iter()
                    .filter(|op| !op.is_observable_only())
                    .collect();
                if !restricted_ops.is_empty() {
                    println!(
                        "  note: {} op(s) require side effects and would be skipped in restricted mode",
                        restricted_ops.len()
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("verdict: REFUSE");
            println!("reason ({}):", classify(&e));
            println!("  {}", explain_one(&e));
            println!();
            println!("structured fields:");
            print_structured(&e);
            ExitCode::from(1)
        }
    }
}

fn run_derive_cmd(args: &[String]) -> ExitCode {
    // Expected: ignis0 derive <input> --rule <rule_id> [--out <path>]
    let input = match args.get(2) {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("Usage: ignis0 derive <input> --rule <rule_id> [--out <path>]");
            return ExitCode::from(2);
        }
    };
    let mut rule: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--rule" => {
                rule = args.get(i + 1).cloned();
                i += 2;
            }
            "--out" => {
                out = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            other => {
                eprintln!("derive: unrecognised argument {:?}", other);
                return ExitCode::from(2);
            }
        }
    }
    let Some(rule) = rule else {
        eprintln!("derive: --rule <id> is required");
        return ExitCode::from(2);
    };
    if rule == ignis0::envelope::GENESIS_RULE {
        eprintln!(
            "derive: --rule {:?} is reserved for top-level forms with no parents",
            rule
        );
        return ExitCode::from(2);
    }
    let bytes = match std::fs::read(&input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("derive: cannot read {}: {}", input.display(), e);
            return ExitCode::from(2);
        }
    };
    let parent = match FormEnvelope::from_json_bytes(&bytes) {
        Ok(env) => env,
        Err(e) => {
            eprintln!("derive: cannot parse parent: {}", e);
            return ExitCode::from(2);
        }
    };
    let child = derive_form(&parent, &rule, None);
    let json = child.to_pretty_json();
    match out {
        Some(p) => {
            if let Err(e) = std::fs::write(&p, json.as_bytes()) {
                eprintln!("derive: cannot write {}: {}", p.display(), e);
                return ExitCode::from(2);
            }
            println!("derive: wrote {}", p.display());
            println!("  child form_id = {}", child.form_id);
            println!("  child hash    = {}", short_hash(&child.hash));
            println!("  proof_status  = {:?} (default)", child.proof_status);
        }
        None => {
            // Stream the new envelope to stdout.
            println!("{}", json);
        }
    }
    ExitCode::SUCCESS
}

// ── Explain helpers ──────────────────────────────────────────────────

fn classify(e: &VerifyError) -> &'static str {
    match e {
        VerifyError::HashMismatch { .. } => "hash mismatch",
        VerifyError::MissingParent { .. } => "missing parent",
        VerifyError::OrphanForm { .. } => "missing parent",
        VerifyError::InvalidProofStatus => "invalid proof state",
        VerifyError::UndeclaredCapability { .. } => "undeclared capability",
        VerifyError::UnresolvedObligations { .. } => "unresolved obligations",
    }
}

fn explain_one(e: &VerifyError) -> String {
    format!("{}", e)
}

fn print_structured(e: &VerifyError) {
    match e {
        VerifyError::HashMismatch { expected, actual } => {
            println!("  kind     = hash_mismatch");
            println!("  expected = {}", expected);
            println!("  actual   = {}", actual);
        }
        VerifyError::MissingParent { parent } => {
            println!("  kind     = missing_parent");
            println!("  parent   = {}", parent);
        }
        VerifyError::OrphanForm { rule } => {
            println!("  kind     = orphan_form");
            println!("  rule     = {}", rule);
            println!("  hint     = use rule == \"genesis\" with parents == []");
        }
        VerifyError::InvalidProofStatus => {
            println!("  kind     = invalid_proof_status");
            println!("  hint     = the proof checker refused this form; do not run");
        }
        VerifyError::UndeclaredCapability {
            op_index,
            op_name,
            required,
            declared,
        } => {
            println!("  kind     = undeclared_capability");
            println!("  op_index = {}", op_index);
            println!("  op_name  = {}", op_name);
            println!("  required = {}", required);
            println!("  declared = {:?}", declared);
        }
        VerifyError::UnresolvedObligations { obligations } => {
            println!("  kind     = unresolved_obligations");
            println!("  count    = {}", obligations.len());
            for (i, o) in obligations.iter().enumerate() {
                println!("  obligation[{}] = {}", i, o);
            }
        }
    }
}

fn short_hash(h: &str) -> String {
    if h.len() <= 16 {
        h.to_string()
    } else {
        format!("{}…{}", &h[..8], &h[h.len() - 4..])
    }
}

// Tiny extension trait so the verify command can print a friendly
// proof-status label without depending on Display for ProofStatus.
trait ProofStatusLabel {
    fn proof_status_label(&self) -> &'static str;
}

impl ProofStatusLabel for ignis0::verify::VerifyOutcome {
    fn proof_status_label(&self) -> &'static str {
        match self.proof_status {
            ignis0::envelope::ProofStatus::Verified => "verified → full execution",
            ignis0::envelope::ProofStatus::Deferred => "deferred → restricted execution",
            ignis0::envelope::ProofStatus::Invalid => "invalid → execution denied",
        }
    }
}
