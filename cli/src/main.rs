use std::env;

struct CliConfig {
    node_name: Option<String>,
    selected_peers: Option<String>,
    target_peers: Option<String>,
}

fn print_help() {
    println!("clippy-share cli");
    println!();
    println!("Usage:");
    println!("  cli [--node-name <name>] [--selected-peers <peer1,peer2>] [--target-peers <peer1,peer2>]");
    println!();
    println!("Flags:");
    println!("  --node-name       Friendly node name announced in clipboard messages");
    println!("  --selected-peers  Local auto-apply allowlist (peer IDs)");
    println!("  --target-peers    Outbound target peer IDs for clipboard broadcasts");
    println!("  -h, --help        Show this help");
}

fn parse_args() -> Result<CliConfig, String> {
    let mut args = env::args().skip(1);

    let mut node_name = None;
    let mut selected_peers = None;
    let mut target_peers = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--node-name" => {
                let value = args
                    .next()
                    .ok_or_else(|| "Missing value for --node-name".to_string())?;
                node_name = Some(value);
            }
            "--selected-peers" => {
                let value = args
                    .next()
                    .ok_or_else(|| "Missing value for --selected-peers".to_string())?;
                selected_peers = Some(value);
            }
            "--target-peers" => {
                let value = args
                    .next()
                    .ok_or_else(|| "Missing value for --target-peers".to_string())?;
                target_peers = Some(value);
            }
            unknown => {
                return Err(format!("Unknown argument: {unknown}"));
            }
        }
    }

    Ok(CliConfig {
        node_name,
        selected_peers,
        target_peers,
    })
}

fn apply_cli_env(config: &CliConfig) {
    // Safety: this runs before any threads are started in this process.
    if let Some(value) = &config.node_name {
        unsafe {
            env::set_var("CLIPPY_NODE_NAME", value);
        }
    }

    if let Some(value) = &config.selected_peers {
        unsafe {
            env::set_var("CLIPPY_SELECTED_PEERS", value);
        }
    }

    if let Some(value) = &config.target_peers {
        unsafe {
            env::set_var("CLIPPY_TARGET_PEERS", value);
        }
    }
}

#[tokio::main]
async fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Argument error: {err}");
            print_help();
            std::process::exit(2);
        }
    };

    apply_cli_env(&config);

    println!("Starting daemon...");
    if let Some(name) = &config.node_name {
        println!("Node name: {name}");
    }
    if let Some(selected) = &config.selected_peers {
        println!("Selected-peers mode enabled for: {selected}");
    }
    if let Some(targets) = &config.target_peers {
        println!("Outbound target peers: {targets}");
    }

    let daemon = core_daemon::CoreDaemon::new();
    daemon.run().await;
}
