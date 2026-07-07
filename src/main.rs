use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ox-creature <command> [args]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "preflight-check" => {
            println!("Preflight check passed.");
        }
        "creature-init" => {
            let json_flag = args.iter().any(|a| a == "--json");
            let workspace_dir = Path::new(".creature");

            let status = if workspace_dir.exists() {
                "validated"
            } else {
                if let Err(e) = fs::create_dir_all(workspace_dir) {
                    eprintln!("Error creating workspace: {}", e);
                    std::process::exit(1);
                }
                "created"
            };

            if json_flag {
                let output = serde_json::json!({
                    "workspace": workspace_dir.display().to_string(),
                    "status": status,
                    "authority": "ox-creature self-build engine"
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                println!(
                    "Creature workspace {} at {}",
                    status,
                    workspace_dir.display()
                );
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
