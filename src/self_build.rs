use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::generated_self_cells;

const CONFIG_PATH: &str = "config/self-build.json";
const STATE_PATH: &str = "state/self-build.json";
const RESULT_PATH: &str = "state/self-build-result.json";
const DEFAULT_DELAY_SECONDS: u64 = 0;

const SECRET_MARKERS: &[&str] = &[
    "sk-",
    "ghp_",
    "github_pat_",
    "BEGIN PRIVATE KEY",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfBuildConfig {
    pub enabled: bool,
    pub loop_delay_seconds: u64,
    pub max_cycles: u64,
    pub max_files_per_cycle: usize,
    pub max_bytes_per_file: usize,
    pub allowed_path_prefixes: Vec<String>,
    pub forbidden_path_prefixes: Vec<String>,
    pub judgment_ready_marker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfBuildState {
    pub cycle: u64,
    pub phase: String,
    pub status: String,
    pub last_summary: String,
    pub ready_for_judgment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMutationAction {
    pub decision: String,
    pub summary: String,
    pub commit_message: Option<String>,
    pub delay_seconds: Option<u64>,
    #[serde(default)]
    pub files: Vec<MutationFile>,
    pub experience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationFile {
    pub path: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfBuildResult {
    pub status: String,
    pub cycle: u64,
    pub summary: String,
    pub commit_message: String,
    pub should_continue: bool,
    pub ready_for_judgment: bool,
    pub changed_files: Vec<String>,
    pub delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
struct TargetCell {
    id: String,
    purpose: String,
    proof_path: String,
    status: String,
}

impl Default for SelfBuildConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            loop_delay_seconds: DEFAULT_DELAY_SECONDS,
            max_cycles: 200,
            max_files_per_cycle: 3,
            max_bytes_per_file: 50_000,
            allowed_path_prefixes: vec![
                "apps/".to_string(),
                "config/".to_string(),
                "contracts/".to_string(),
                "docs/".to_string(),
                "flows/".to_string(),
                "src/".to_string(),
                "state/".to_string(),
            ],
            forbidden_path_prefixes: vec![".git/".to_string(), "target/".to_string()],
            judgment_ready_marker: "state/judgment-ready.json".to_string(),
        }
    }
}

impl Default for SelfBuildState {
    fn default() -> Self {
        Self {
            cycle: 0,
            phase: "genesis".to_string(),
            status: "self-build-ready".to_string(),
            last_summary: "Self-build has not run yet.".to_string(),
            ready_for_judgment: false,
        }
    }
}

pub fn config_check() -> Result<(), String> {
    let config = load_config()?;
    if !config.enabled {
        return Err("self-build is disabled in config/self-build.json".to_string());
    }
    if config.max_cycles == 0 {
        return Err("self-build max_cycles must be greater than zero".to_string());
    }
    if config.max_files_per_cycle == 0 {
        return Err("self-build max_files_per_cycle must be greater than zero".to_string());
    }
    if config.max_bytes_per_file == 0 {
        return Err("self-build max_bytes_per_file must be greater than zero".to_string());
    }
    println!("self-build-config-check: ok");
    Ok(())
}

pub fn self_build_rate_seconds() -> Result<(), String> {
    let config = load_config()?;
    println!("{}", config.loop_delay_seconds);
    Ok(())
}

pub fn self_build_context(args: &[String]) -> Result<(), String> {
    if !args.iter().any(|arg| arg == "--json") {
        return Err("self-build-context requires --json".to_string());
    }
    let config = load_config()?;
    let state = load_state()?;
    let target_cells = target_cells(&config);
    let missing_cells: Vec<TargetCell> = target_cells
        .iter()
        .filter(|cell| cell.status != "present")
        .cloned()
        .collect();
    let seed = read_optional("SEED.md");
    let experience = read_optional("EXPERIENCE.md");
    let tasks = read_optional("current_tasks.txt");

    let context = json!({
        "project": "ox-creature",
        "role": "self-build-context",
        "meaning": "This is not Judgment Day. This is the self-build loop. The creature should make one tiny safe mutation toward the product, commit it, and continue.",
        "laws": [
            "Human Sovereignty",
            "Reality Before Meaning",
            "LLM Is Not Authority",
            "Flow Before Code",
            "Git Is the Outer Memory",
            "Cost Gravity",
            "Failure Becomes Experience",
            "Small Steps or Stop"
        ],
        "config": config,
        "state": state,
        "generated_heartbeat": {
            "cycle": generated_self_cells::SELF_BUILD_CYCLE,
            "last_event": generated_self_cells::LAST_SELF_BUILD_EVENT
        },
        "target_product": {
            "definition": "A tiny Rust runtime + markdown constitution + GitHub Actions + LLM that self-builds until it can turn a user story into code and expose proof to Cockpit, then asks for Judgment Day.",
            "cells": target_cells,
            "missing_cells": missing_cells
        },
        "source_context": {
            "seed_excerpt": truncate(&seed, 5000),
            "experience_excerpt": truncate(&experience, 5000),
            "current_tasks_excerpt": truncate(&tasks, 3000)
        },
        "required_response_contract": {
            "type": "json-only",
            "schema_file": "contracts/llm-next-action.schema.json",
            "allowed_decisions": ["continue", "ready_for_judgment", "sleep", "stop"],
            "rules": [
                "Choose exactly one small missing target cell.",
                "Modify at most max_files_per_cycle files.",
                "Return full file contents as base64.",
                "Never include secrets.",
                "Never mutate .github workflows unless a human patch explicitly changes them.",
                "Do not claim ready_for_judgment until user story intake, materialization, and cockpit proof cells exist."
            ]
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&context).map_err(|err| err.to_string())?
    );
    Ok(())
}

pub fn llm_action_from_response(args: &[String]) -> Result<(), String> {
    let input = args
        .get(2)
        .ok_or_else(|| "usage: llm-action-from-response INPUT_JSON OUTPUT_JSON".to_string())?;
    let output = args
        .get(3)
        .ok_or_else(|| "usage: llm-action-from-response INPUT_JSON OUTPUT_JSON".to_string())?;
    let text = fs::read_to_string(input).map_err(|err| format!("failed to read {input}: {err}"))?;
    let parsed: Value = serde_json::from_str(&text)
        .map_err(|err| format!("LLM response is not valid JSON: {err}"))?;
    let content = parsed
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| "LLM response missing choices[0].message.content".to_string())?;
    let json_text = extract_first_json_object(content)?;
    let action: LlmMutationAction = serde_json::from_str(&json_text)
        .map_err(|err| format!("LLM action content is not valid action JSON: {err}"))?;
    validate_action_shape(&action)?;
    write_json(output, &action)?;
    println!("llm-action-from-response: ok {output}");
    Ok(())
}

pub fn self_build_step(args: &[String]) -> Result<(), String> {
    let config = load_config()?;
    let mut state = load_state()?;
    if !config.enabled {
        return Err("self-build is disabled".to_string());
    }
    if state.cycle >= config.max_cycles {
        return Err(format!(
            "self-build max_cycles reached: {} >= {}",
            state.cycle, config.max_cycles
        ));
    }

    let action_path = read_flag_value(args, "--action");
    let wants_json = args.iter().any(|arg| arg == "--json");
    let action = match action_path {
        Some(path) => match load_action(path) {
            Ok(action) => Some(action),
            Err(err) => {
                record_experience(&format!(
                    "LLM action was rejected; deterministic fallback used. Reason: {err}"
                ))?;
                None
            }
        },
        None => None,
    };

    let result = match action {
        Some(action) => apply_action(action, &config, &mut state)?,
        None => deterministic_fallback(&config, &mut state)?,
    };

    write_json(RESULT_PATH, &result)?;
    if wants_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?
        );
    } else {
        println!("self-build-step: {}", result.summary);
    }
    Ok(())
}

pub fn self_build_ready_check() -> Result<(), String> {
    let config = load_config()?;
    let marker_path = Path::new(&config.judgment_ready_marker);
    if !marker_path.is_file() {
        return Err(format!(
            "not ready for Judgment Day: missing {}",
            config.judgment_ready_marker
        ));
    }
    let required = [
        "contracts/user-story.schema.json",
        "contracts/materialized-patch.schema.json",
        "apps/cockpit/README.md",
        "state/judgment-ready.json",
    ];
    let mut missing = Vec::new();
    for path in required {
        if !Path::new(path).is_file() {
            missing.push(path);
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "not ready for Judgment Day: missing product cells: {}",
            missing.join(", ")
        ));
    }
    println!("self-build-ready-check: ready");
    Ok(())
}

pub fn status_block_json() -> Result<String, String> {
    let state = load_state()?;
    let config = load_config()?;
    let target_cells = target_cells(&config);
    let missing_count = target_cells
        .iter()
        .filter(|cell| cell.status != "present")
        .count();
    let status = json!({
        "project": "ox-creature",
        "route": "cockpit.identity",
        "status": "ready",
        "blocks": [
            {
                "type": "law_panel",
                "data": {
                    "laws": [
                        "Human Sovereignty",
                        "Reality Before Meaning",
                        "LLM Is Not Authority",
                        "Flow Before Code",
                        "Git Is the Outer Memory",
                        "Judgment Day",
                        "Cost Gravity",
                        "Failure Becomes Experience",
                        "Small Steps or Stop"
                    ]
                }
            },
            {
                "type": "self_build_status",
                "data": {
                    "cycle": state.cycle,
                    "phase": state.phase,
                    "status": state.status,
                    "ready_for_judgment": state.ready_for_judgment,
                    "loop_delay_seconds": config.loop_delay_seconds,
                    "missing_product_cells": missing_count,
                    "heartbeat_cycle": generated_self_cells::SELF_BUILD_CYCLE,
                    "heartbeat_event": generated_self_cells::LAST_SELF_BUILD_EVENT
                }
            },
            {
                "type": "runtime_status",
                "data": {
                    "runtime": "tiny-rust",
                    "self_build": "github-actions-mutating-loop",
                    "judgment_day": "only-when-creature-declares-ready",
                    "authority": "human",
                    "llm_authority": false,
                    "llm_model_selection": "runtime_discovery"
                }
            }
        ]
    });
    serde_json::to_string_pretty(&status).map_err(|err| err.to_string())
}

fn apply_action(
    action: LlmMutationAction,
    config: &SelfBuildConfig,
    state: &mut SelfBuildState,
) -> Result<SelfBuildResult, String> {
    validate_action_shape(&action)?;
    let decision = action.decision.as_str();
    let next_cycle = state.cycle + 1;

    if decision == "ready_for_judgment" {
        fs::create_dir_all("state").map_err(|err| format!("failed to create state/: {err}"))?;
        let summary = action.summary.clone();
        let marker = json!({
            "declared_by": "ox-creature-self-build-loop",
            "cycle": next_cycle,
            "summary": summary,
            "meaning": "The creature declares it is ready to request Judgment Day. Runtime will still verify required product cells."
        });
        write_json(&config.judgment_ready_marker, &marker)?;
        state.cycle = next_cycle;
        state.phase = "judgment-requested".to_string();
        state.status = "ready-for-judgment".to_string();
        state.last_summary = action.summary.clone();
        state.ready_for_judgment = true;
        write_json(STATE_PATH, state)?;
        return Ok(SelfBuildResult {
            status: "ready_for_judgment".to_string(),
            cycle: state.cycle,
            summary: action.summary,
            commit_message: action
                .commit_message
                .unwrap_or_else(|| "self-build: request judgment day".to_string()),
            should_continue: false,
            ready_for_judgment: true,
            changed_files: vec![config.judgment_ready_marker.clone(), STATE_PATH.to_string()],
            delay_seconds: action.delay_seconds.unwrap_or(config.loop_delay_seconds),
        });
    }

    if decision == "stop" {
        state.cycle = next_cycle;
        state.phase = "stopped".to_string();
        state.status = "self-build-stopped".to_string();
        state.last_summary = action.summary.clone();
        write_json(STATE_PATH, state)?;
        return Ok(SelfBuildResult {
            status: "stopped".to_string(),
            cycle: state.cycle,
            summary: action.summary,
            commit_message: action
                .commit_message
                .unwrap_or_else(|| "self-build: stop by creature decision".to_string()),
            should_continue: false,
            ready_for_judgment: false,
            changed_files: vec![STATE_PATH.to_string()],
            delay_seconds: action.delay_seconds.unwrap_or(config.loop_delay_seconds),
        });
    }

    if decision == "sleep" {
        state.cycle = next_cycle;
        state.phase = "sleep".to_string();
        state.status = "self-build-sleep".to_string();
        state.last_summary = action.summary.clone();
        write_json(STATE_PATH, state)?;
        return Ok(SelfBuildResult {
            status: "sleep".to_string(),
            cycle: state.cycle,
            summary: action.summary,
            commit_message: action
                .commit_message
                .unwrap_or_else(|| "self-build: sleep".to_string()),
            should_continue: true,
            ready_for_judgment: false,
            changed_files: vec![STATE_PATH.to_string()],
            delay_seconds: action.delay_seconds.unwrap_or(config.loop_delay_seconds),
        });
    }

    if decision != "continue" {
        return Err(format!("unsupported self-build decision: {decision}"));
    }

    if action.files.is_empty() {
        return deterministic_fallback(config, state);
    }
    if action.files.len() > config.max_files_per_cycle {
        return Err(format!(
            "mutation touches too many files: {} > {}",
            action.files.len(), config.max_files_per_cycle
        ));
    }

    let mut changed = Vec::new();
    for file in &action.files {
        let path = validate_mutation_path(&file.path, config)?;
        let bytes = BASE64_STANDARD.decode(&file.content_base64).map_err(|err| {
            format!("failed to decode base64 for {}: {err}", file.path)
        })?;
        if bytes.len() > config.max_bytes_per_file {
            return Err(format!(
                "{} exceeds max_bytes_per_file: {} > {}",
                file.path,
                bytes.len(),
                config.max_bytes_per_file
            ));
        }
        let text = String::from_utf8(bytes)
            .map_err(|err| format!("{} is not valid UTF-8: {err}", file.path))?;
        scan_for_secret_markers(&file.path, &text)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        fs::write(&path, text).map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        changed.push(file.path.clone());
    }

    state.cycle = next_cycle;
    state.phase = "self-building".to_string();
    state.status = "mutation-applied".to_string();
    state.last_summary = action.summary.clone();
    state.ready_for_judgment = false;
    write_json(STATE_PATH, state)?;
    changed.push(STATE_PATH.to_string());

    Ok(SelfBuildResult {
        status: "mutated".to_string(),
        cycle: state.cycle,
        summary: action.summary,
        commit_message: action
            .commit_message
            .unwrap_or_else(|| format!("self-build: cycle {}", state.cycle)),
        should_continue: true,
        ready_for_judgment: false,
        changed_files: changed,
        delay_seconds: action.delay_seconds.unwrap_or(config.loop_delay_seconds),
    })
}

fn deterministic_fallback(
    config: &SelfBuildConfig,
    state: &mut SelfBuildState,
) -> Result<SelfBuildResult, String> {
    let next_cycle = state.cycle + 1;
    fs::create_dir_all("state/self-build-cycles")
        .map_err(|err| format!("failed to create cycle directory: {err}"))?;
    let cycle_path = format!("state/self-build-cycles/cycle-{next_cycle:06}.md");
    let summary = format!(
        "Self-build cycle {next_cycle}: no valid LLM patch was available, so the runtime advanced by deterministic heartbeat."
    );
    fs::write(
        &cycle_path,
        format!(
            "# Self-build cycle {next_cycle}\n\n{summary}\n\nThis is a fallback mutation. The next loop should try to use LLM output again.\n"
        ),
    )
    .map_err(|err| format!("failed to write {cycle_path}: {err}"))?;

    let generated = format!(
        "// This file is intentionally mutated by the ox-creature self-build loop.\n// It is a tiny deterministic heartbeat proving that the creature can change itself,\n// commit the change, and continue through GitHub Actions.\n\npub const SELF_BUILD_CYCLE: u64 = {next_cycle};\npub const LAST_SELF_BUILD_EVENT: &str = \"deterministic-fallback-cycle-{next_cycle}\";\n"
    );
    fs::write("src/generated_self_cells.rs", generated)
        .map_err(|err| format!("failed to write src/generated_self_cells.rs: {err}"))?;

    state.cycle = next_cycle;
    state.phase = "self-building".to_string();
    state.status = "deterministic-fallback".to_string();
    state.last_summary = summary.clone();
    state.ready_for_judgment = false;
    write_json(STATE_PATH, state)?;

    Ok(SelfBuildResult {
        status: "fallback-mutated".to_string(),
        cycle: state.cycle,
        summary,
        commit_message: format!("self-build: fallback cycle {}", state.cycle),
        should_continue: state.cycle < config.max_cycles,
        ready_for_judgment: false,
        changed_files: vec![
            cycle_path,
            "src/generated_self_cells.rs".to_string(),
            STATE_PATH.to_string(),
        ],
        delay_seconds: config.loop_delay_seconds,
    })
}

fn validate_action_shape(action: &LlmMutationAction) -> Result<(), String> {
    match action.decision.as_str() {
        "continue" | "ready_for_judgment" | "sleep" | "stop" => {}
        other => return Err(format!("invalid decision: {other}")),
    }
    if action.summary.trim().is_empty() {
        return Err("action.summary is required".to_string());
    }
    if let Some(commit_message) = &action.commit_message {
        if commit_message.trim().is_empty() {
            return Err("commit_message must not be empty".to_string());
        }
        if commit_message.len() > 120 {
            return Err("commit_message must be <= 120 characters".to_string());
        }
    }
    Ok(())
}

fn validate_mutation_path(path: &str, config: &SelfBuildConfig) -> Result<PathBuf, String> {
    if path.trim().is_empty() {
        return Err("mutation file path is empty".to_string());
    }
    let raw = Path::new(path);
    if raw.is_absolute() {
        return Err(format!("absolute paths are forbidden: {path}"));
    }
    for component in raw.components() {
        match component {
            Component::ParentDir => return Err(format!("path traversal is forbidden: {path}")),
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("rooted paths are forbidden: {path}"))
            }
            _ => {}
        }
    }
    if config
        .forbidden_path_prefixes
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return Err(format!("forbidden mutation path: {path}"));
    }
    if path.starts_with(".github/") {
        return Err("self-build may not mutate .github workflows; use a human patch".to_string());
    }
    if !config
        .allowed_path_prefixes
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return Err(format!("path is outside allowed mutation prefixes: {path}"));
    }
    Ok(PathBuf::from(path))
}

fn target_cells(config: &SelfBuildConfig) -> Vec<TargetCell> {
    vec![
        cell(
            "runtime.self_build_engine",
            "Runtime can produce context, accept LLM patch JSON, validate it, mutate files, and update state.",
            "src/self_build.rs",
        ),
        cell(
            "runtime.self_build_rate",
            "Self-build loop delay is configurable in seconds and defaults to zero.",
            CONFIG_PATH,
        ),
        cell(
            "contract.llm_next_action",
            "LLM output contract exists and constrains mutations to small validated patches.",
            "contracts/llm-next-action.schema.json",
        ),
        cell(
            "github.self_build_loop",
            "GitHub Actions can run the self-build cycle, commit, push, and dispatch continuation.",
            ".github/workflows/self-loop.yml",
        ),
        cell(
            "user_story.intake_contract",
            "A user story has a typed contract before it can become code.",
            "contracts/user-story.schema.json",
        ),
        cell(
            "materialization.patch_contract",
            "Code materialization is represented as a bounded patch artifact before execution.",
            "contracts/materialized-patch.schema.json",
        ),
        cell(
            "cockpit.surface",
            "Cockpit has a visible surface that can show creature state, flow, diffs, risks, and Judgment proof.",
            "apps/cockpit/README.md",
        ),
        cell(
            "judgment.readiness_marker",
            "The creature can stop the self-build loop and request Judgment Day only after product cells exist.",
            &config.judgment_ready_marker,
        ),
    ]
}

fn cell(id: &str, purpose: &str, proof_path: &str) -> TargetCell {
    let status = if Path::new(proof_path).is_file() {
        "present"
    } else {
        "missing"
    };
    TargetCell {
        id: id.to_string(),
        purpose: purpose.to_string(),
        proof_path: proof_path.to_string(),
        status: status.to_string(),
    }
}

fn load_config() -> Result<SelfBuildConfig, String> {
    let path = Path::new(CONFIG_PATH);
    if !path.is_file() {
        return Ok(SelfBuildConfig::default());
    }
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {CONFIG_PATH}: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid {CONFIG_PATH}: {err}"))
}

fn load_state() -> Result<SelfBuildState, String> {
    let path = Path::new(STATE_PATH);
    if !path.is_file() {
        return Ok(SelfBuildState::default());
    }
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {STATE_PATH}: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid {STATE_PATH}: {err}"))
}

fn load_action(path: &str) -> Result<LlmMutationAction, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid action JSON {path}: {err}"))
}

fn read_flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn write_json<T: Serialize>(path: &str, value: &T) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
    }
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(path, format!("{text}\n")).map_err(|err| format!("failed to write {path}: {err}"))
}

fn record_experience(message: &str) -> Result<(), String> {
    let path = "EXPERIENCE.md";
    let mut text = read_optional(path);
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("\n## Self-build experience\n\n");
    text.push_str(message);
    text.push('\n');
    fs::write(path, text).map_err(|err| format!("failed to update {path}: {err}"))
}

fn read_optional(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}\n...[truncated]", &text[..max])
    }
}

fn extract_first_json_object(text: &str) -> Result<String, String> {
    let text = text.trim();
    let text = text
        .strip_prefix("```json")
        .or_else(|| text.strip_prefix("```"))
        .unwrap_or(text)
        .trim();
    let text = text.strip_suffix("```").unwrap_or(text).trim();
    if text.starts_with('{') && text.ends_with('}') {
        return Ok(text.to_string());
    }

    let mut start = None;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let begin = start.ok_or_else(|| "internal JSON extraction error".to_string())?;
                    return Ok(text[begin..=idx].to_string());
                }
            }
            _ => {}
        }
    }
    Err("could not find JSON object in LLM response content".to_string())
}

fn scan_for_secret_markers(path: &str, text: &str) -> Result<(), String> {
    for marker in SECRET_MARKERS {
        if text.contains(marker) {
            return Err(format!("{path} contains possible secret marker: {marker}"));
        }
    }
    Ok(())
}
