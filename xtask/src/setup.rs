use std::io;
use std::process::Command;

struct Check {
    name: &'static str,
    args: &'static [&'static str],
    required: bool,
    install_hint: &'static str,
}

const CHECKS: &[Check] = &[
    Check {
        name: "rustc",
        args: &["rustc", "--version"],
        required: true,
        install_hint: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
    },
    Check {
        name: "cargo",
        args: &["cargo", "--version"],
        required: true,
        install_hint: "Installed with rustup (see above)",
    },
    Check {
        name: "rustfmt",
        args: &["rustfmt", "--version"],
        required: false,
        install_hint: "rustup component add rustfmt",
    },
    Check {
        name: "clippy-driver",
        args: &["clippy-driver", "--version"],
        required: false,
        install_hint: "rustup component add clippy",
    },
    Check {
        name: "cargo-watch",
        args: &["cargo-watch", "--version"],
        required: false,
        install_hint: "cargo install cargo-watch",
    },
];

pub fn run() -> Result<(), io::Error> {
    println!("Checking environment...\n");
    let mut all_good = true;

    for check in CHECKS {
        let found = check_exists(check.args);
        let status = if found {
            "\x1b[32m✓\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };
        let label = if check.required {
            "required"
        } else {
            "recommended"
        };

        println!("  {status} {:<14} ({label})", check.name);

        if !found {
            println!("    → Install with: {}", check.install_hint);
            if check.required {
                all_good = false;
            }
        }
    }

    println!();
    if all_good {
        println!("Ready! Run `cargo solve` to start a new problem.");
    } else {
        println!("Some required tools are missing. Install them and run `cargo setup` again.");
    }
    Ok(())
}

fn check_exists(args: &[&str]) -> bool {
    Command::new(args[0])
        .args(&args[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
