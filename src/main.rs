mod generated_self_cells;
mod self_build;

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const REQUIRED_LAWS: &[&str] = &[
    "Human Sovereignty",
    "Reality Before Meaning",
    "LLM Is Not Authority",
    "Flow Before Code",
    "Git Is the Outer Memory",
    "Judgment Day",
    "Cost Gravity",
    "Failure Becomes Experience",
    "Small Steps or Stop",
];

const SECRET_MARKERS: &[&str] = &[
    "sk-",
    "ghp_",
    "github_pat_",
    "BEGIN PRIVATE KEY",
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("help");
    let result = match command {
        "seed-check" => seed_check(),
        "flow-check" => flow_check(),
        "contract-check" => contract_check(),
        "llm-config-check" => llm_config_check(),
        "status" => status_json(),
        "preflight-check" => preflight_check(),
        "self-build-rate-seconds" => self_build::self_build_rate_seconds(),
        "self-build-context" => self_build::self_build_context(&args),
        "llm-action-from-response" => self_build::llm_action_from_response(&args),
        "self-build-step" => self_build::self_build_step(&args),
        "self-build-ready-check" => self_build::self_build_ready_check(),
        "help" | "--help" | "-h" => help(),
        other => Err(format!("unknown command: {other}")),
    };

    if let Err(message) = result {
        eprintln!("error: {message}");
        process::exit(1);
    }
}

fn help() -> Result<(), String> {
    println!("ox-creature Runtime");
    println!("commands:");
    println!("  seed-check                    validate permanent law and secret hygiene");
    println!("  flow-check                    validate flow files and node references");
    println!("  contract-check                validate contract files exist and look like JSON schemas");
    println!("  llm-config-check              validate non-secret LLM config and model discovery policy");
    println!("  status --json                 emit Cockpit-readable presentation blocks");
    println!("  preflight-check               run local checks; not Judgment Day");
    println!("  self-build-rate-seconds       print loop delay in seconds; default is 0");
    println!("  self-build-context --json     emit bounded context for the LLM self-build step");
    println!("  llm-action-from-response IN OUT  extract JSON action from OpenAI-compatible response");
    println!("  self-build-step [--action FILE] --json  apply one governed self-build mutation");
    println!("  self-build-ready-check        fail unless product cells are ready for Judgment Day");
    Ok(())
}

fn seed_check() -> Result<(), String> {
    let seed = read_file("SEED.md")?;
    let mut missing = Vec::new();
    for law in REQUIRED_LAWS {
        if !seed.contains(law) {
            missing.push(*law);
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "SEED.md missing required laws: {}",
            missing.join(", ")
        ));
    }
    scan_for_secret_markers("SEED.md", &seed)?;
    println!("seed-check: ok");
    Ok(())
}

fn flow_check() -> Result<(), String> {
    let flow_dir = Path::new("flows");
    if !flow_dir.is_dir() {
        return Err("missing flows/ directory".to_string());
    }
    let mut checked = 0usize;
    for path in read_dir_sorted(flow_dir)? {
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let text = read_path(&path)?;
        require_contains(&text, "\"flow_id\"", &path)?;
        require_contains(&text, "\"nodes\"", &path)?;
        require_contains(&text, "\"forbidden\"", &path)?;
        validate_flow_references(&text, &path)?;
        checked += 1;
        println!("flow-check: ok {}", path.display());
    }
    if checked == 0 {
        return Err("no JSON flow files found".to_string());
    }
    Ok(())
}

fn contract_check() -> Result<(), String> {
    let contract_dir = Path::new("contracts");
    if !contract_dir.is_dir() {
        return Err("missing contracts/ directory".to_string());
    }
    let mut checked = 0usize;
    for path in read_dir_sorted(contract_dir)? {
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let text = read_path(&path)?;
        require_contains(&text, "\"$schema\"", &path)?;
        require_contains(&text, "\"type\"", &path)?;
        checked += 1;
        println!("contract-check: ok {}", path.display());
    }
    if checked == 0 {
        return Err("no JSON contract files found".to_string());
    }
    Ok(())
}

fn preflight_check() -> Result<(), String> {
    seed_check()?;
    contract_check()?;
    flow_check()?;
    llm_config_check()?;
    self_build::config_check()?;
    status_json()?;
    println!("preflight-check: ok");
    Ok(())
}

fn llm_config_check() -> Result<(), String> {
    let path = Path::new("config/llm.runtime.json");
    let text = read_path(path)?;
    require_contains(&text, "\"provider\"", path)?;
    require_contains(&text, "\"base_url\"", path)?;
    require_contains(&text, "\"model_discovery\"", path)?;
    require_contains(&text, "\"api_key_secret_name\"", path)?;
    require_contains(&text, "LLM_API_KEY", path)?;
    scan_for_secret_markers("config/llm.runtime.json", &text)?;
    if text.contains("put-your-key-here") {
        return Err("config/llm.runtime.json must not contain placeholder secrets".to_string());
    }
    println!("llm-config-check: ok");
    Ok(())
}

fn status_json() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let wants_json = args.iter().any(|arg| arg == "--json");
    if !wants_json {
        println!("status: ready");
        println!("run with --json for Cockpit blocks");
        return Ok(());
    }

    let status = self_build::status_block_json()?;
    println!("{}", status);
    Ok(())
}

fn validate_flow_references(text: &str, path: &Path) -> Result<(), String> {
    let node_ids = collect_json_string_values_after_key(text, "id");
    let requires = collect_requires_values(text);
    for required in requires {
        if !node_ids.contains(&required) {
            return Err(format!(
                "{} references missing node id in requires: {}",
                path.display(),
                required
            ));
        }
    }
    Ok(())
}

fn collect_requires_values(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let bytes = text.as_bytes();
    let key = b"\"requires\"";
    let mut i = 0usize;
    while let Some(pos) = find_subslice(&bytes[i..], key) {
        i += pos + key.len();
        while i < bytes.len() && bytes[i] != b'[' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        i += 1;
        while i < bytes.len() && bytes[i] != b']' {
            if bytes[i] == b'\"' {
                if let Some((value, next)) = read_json_string(bytes, i) {
                    values.push(value);
                    i = next;
                    continue;
                }
            }
            i += 1;
        }
    }
    values
}

fn collect_json_string_values_after_key(text: &str, key_name: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let bytes = text.as_bytes();
    let key = format!("\"{}\"", key_name);
    let key_bytes = key.as_bytes();
    let mut i = 0usize;
    while let Some(pos) = find_subslice(&bytes[i..], key_bytes) {
        i += pos + key_bytes.len();
        while i < bytes.len() && bytes[i] != b':' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'\"' {
            if let Some((value, next)) = read_json_string(bytes, i) {
                values.insert(value);
                i = next;
                continue;
            }
        }
    }
    values
}

fn read_json_string(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    if bytes.get(start) != Some(&b'\"') {
        return None;
    }
    let mut value = String::new();
    let mut escaped = false;
    let mut i = start + 1;
    while i < bytes.len() {
        let byte = bytes[i];
        if escaped {
            value.push(byte as char);
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'\"' {
            return Some((value, i + 1));
        } else {
            value.push(byte as char);
        }
        i += 1;
    }
    None
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn require_contains(text: &str, needle: &str, path: &Path) -> Result<(), String> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(format!(
            "{} missing required marker {}",
            path.display(),
            needle
        ))
    }
}

fn scan_for_secret_markers(path: &str, text: &str) -> Result<(), String> {
    for marker in SECRET_MARKERS {
        if text.contains(marker) {
            return Err(format!("{path} contains possible secret marker: {marker}"));
        }
    }
    Ok(())
}

fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))
}

fn read_path(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
}

fn read_dir_sorted(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for entry in
        fs::read_dir(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_node_ids() {
        let text = r#"{"nodes":[{"id":"a"},{"id":"b"}]}"#;
        let ids = collect_json_string_values_after_key(text, "id");
        assert!(ids.contains("a"));
        assert!(ids.contains("b"));
    }

    #[test]
    fn collects_requires_values() {
        let text = r#"{"nodes":[{"id":"b","requires":["a"]}]}"#;
        assert_eq!(collect_requires_values(text), vec!["a".to_string()]);
    }
}
