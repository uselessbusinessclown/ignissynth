//! ignis0 CLI — a thin wrapper around the library.
//!
//! Usage:
//!   ignis0 fixed-point          Run the A9.3 fixed-point check.
//!   ignis0 pretty-print <file>  Parse a .form source file and
//!                               pretty-print the opcode sequence.
//!   ignis0 version              Print the scaffold version.
//!   ignis0 help                 Print this message.

use ignis0::fixed_point::{FixedPointCheck, FixedPointVerdict};
use ignis0::parser::parse_form_lines;
use ignis0::pretty::pretty_print;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");

    match cmd {
        "fixed-point" => run_fixed_point(),
        "pretty-print" => run_pretty_print(&args),
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
             ignis0 version              Print the scaffold version\n\
             ignis0 help                 Print this message\n\
         \n\
         See ../kernel/IGNITION-BOOTSTRAP.md for the full contract."
    );
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
        FixedPointVerdict::Pass { direct, indirect_1, indirect_2 } => {
            println!("fixed-point: PASS");
            println!("  direct     = {:?}", direct);
            println!("  indirect_1 = {:?}", indirect_1);
            println!("  indirect_2 = {:?}", indirect_2);
            println!("\nA9.3 necessary condition holds. ignis0 is faithful to");
            println!("the IL on the canonical case.");
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
