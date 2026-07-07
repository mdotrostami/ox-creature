use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::generated_self_cells;

const CONFIG_PATH: &str = "config/self-build.json";
const STATE_PATH: &str = "state/self-build.json";
const RESULT_PATH: &str = "state/self-build-result.json";
const TASKS_PATH: &str = "current_tasks.txt";
const DEFAULT_DELAY_SECONDS: u64 = 0;

const SECRET_MARKERS: &[&str] = &["sk-", "ghp_", "github_pat_", "BEGIN PRIVATE KEY"];

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
    pub task_id: Option<String>,
    pub cell_id: Option<String>,
    #[serde(default)]
    pub files: Vec<MutationFile>,
    pub experience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationFile {
    pub path: String,
    pub content_base64: Option<String>,
    pub content: Option<String>,
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
    proof: String,
}

#[derive(Debug, Clone, Serialize)]
struct SelfBuildTask {
    id: String,
    title: String,
    status: String,
    goal: String,
    allowed_files: Vec<String>,
    validation: Vec<String>,
}

#[derive(Debug, Clone)]
struct FileBackup {
    path: String,
    previous: Option<String>,
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
    if current_task()?.is_none() && !all_tasks_done()? {
        return Err("current_tasks.txt has no [NEXT] task but still has [TODO] tasks".to_string());
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
    let current_task = current_task()?;
    let all_done = all_tasks_done()?;
    let tasks = read_optional(TASKS_PATH);
    let seed = read_optional("SEED.md");
    let experience = read_optional("EXPERIENCE.md");

    let context = json!({
        "project": "ox-creature",
        "role": "self-build-context",
        "meaning": "This is the autonomous self-build loop, not Judgment Day. The creature must complete exactly the current [NEXT] task, validate it, commit it, and continue until product readiness is true.",
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
        "hard_boundaries": [
            "The current task ledger is the driver. Do not repeat old product-cell examples.",
            "Return exactly one JSON object and no markdown.",
            "For decision=continue, task_id must equal current_task.id.",
            "Only mutate files listed in current_task.allowed_files.",
            "Never mutate current_tasks.txt, state/self-build.json, state/self-build-result.json, .github workflows, secrets, target/, or .git/.",
            "The runtime advances current_tasks.txt after validation succeeds; the LLM must not edit the task ledger.",
            "The runtime may run deterministic repair such as rustfmt before validation; the LLM still owns the semantic patch.",
            "If the task cannot be safely done, use decision=sleep or stop with a precise reason; do not fake progress.",
            "Do not declare ready_for_judgment while any task remains or any product proof is weak."
        ],
        "config": config,
        "state": state,
        "generated_heartbeat": {
            "cycle": generated_self_cells::SELF_BUILD_CYCLE,
            "last_event": generated_self_cells::LAST_SELF_BUILD_EVENT
        },
        "task_driver": {
            "all_tasks_done": all_done,
            "current_task": current_task,
            "ledger_excerpt": truncate(&tasks, 6000)
        },
        "target_product": {
            "definition": "A tiny Rust runtime + markdown constitution + GitHub Actions + LLM that self-builds until it can turn a user story into code and expose proof to Cockpit, then asks for Judgment Day.",
            "cells": target_cells
        },
        "source_context": {
            "seed_excerpt": truncate(&seed, 5000),
            "experience_excerpt": truncate(&experience, 5000)
        },
        "required_response_contract": {
            "type": "json-only",
            "schema_file": "contracts/llm-next-action.schema.json",
            "allowed_decisions": ["continue", "ready_for_judgment", "sleep", "stop"],
            "minimum_valid_continue": {
                "decision": "continue",
                "task_id": "COPY_CURRENT_TASK_ID_EXACTLY",
                "summary": "Complete the current [NEXT] task with the smallest valid source mutation.",
                "commit_message": "self-build: complete task TASK_ID",
                "files": [
                    {
                        "path": "ONE_ALLOWED_FILE_FROM_CURRENT_TASK",
                        "content": "FULL FILE CONTENT AS PLAIN UTF-8 TEXT"
                    }
                ]
            },
            "rules": [
                "Use the current_task object, not target_product examples, to choose work.",
                "Return full file contents, not a diff.",
                "Prefer the plain content field for UTF-8 files.",
                "Keep changes small and task-scoped.",
                "No secret markers.",
                "No placeholder-only files unless the task explicitly asks for a placeholder.",
                "No no-op rewrites. The changed file must materially satisfy the task goal."
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
    fs::write("state/llm-response-content.txt", truncate(content, 8000))
        .map_err(|err| format!("failed to write state/llm-response-content.txt: {err}"))?;
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
    let result = match action_path {
        Some(path) => match load_action(path) {
            Ok(action) => match apply_action(action, &config, &mut state) {
                Ok(result) => result,
                Err(err) => rejection_cycle(
                    &config,
                    &mut state,
                    format!("LLM action rejected before commit: {err}"),
                )?,
            },
            Err(err) => rejection_cycle(
                &config,
                &mut state,
                format!("failed to load LLM action: {err}"),
            )?,
        },
        None => rejection_cycle(
            &config,
            &mut state,
            "no valid LLM action was available for this self-build cycle".to_string(),
        )?,
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
    if !all_tasks_done()? {
        return Err("not ready for Judgment Day: current_tasks.txt still has NEXT/TODO work".to_string());
    }
    let invalid_cells = invalid_product_cells(&config, true)?;
    if !invalid_cells.is_empty() {
        return Err(format!(
            "not ready for Judgment Day: invalid cells: {}",
            invalid_cells.join(", ")
        ));
    }
    let marker = read_optional(&config.judgment_ready_marker);
    for required in [
        "user_story_to_code_proof",
        "cockpit_proof",
        "validation_proof",
        "creator_judgment_requested",
    ] {
        if !marker.contains(required) {
            return Err(format!(
                "not ready for Judgment Day: marker missing proof field {required}"
            ));
        }
    }
    println!("self-build-ready-check: ready");
    Ok(())
}

pub fn status_block_json() -> Result<String, String> {
    let state = load_state()?;
    let config = load_config()?;
    let target_cells = target_cells(&config);
    let invalid_count = target_cells
        .iter()
        .filter(|cell| cell.status != "valid")
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
                    "invalid_product_cells": invalid_count,
                    "current_task": current_task()?,
                    "heartbeat_cycle": generated_self_cells::SELF_BUILD_CYCLE,
                    "heartbeat_event": generated_self_cells::LAST_SELF_BUILD_EVENT
                }
            },
            {
                "type": "runtime_status",
                "data": {
                    "runtime": "tiny-rust",
                    "self_build": "task-driven-github-actions-mutating-loop",
                    "judgment_day": "only-when-creature-declares-ready-after-proof",
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
    let current = current_task()?;

    if decision == "ready_for_judgment" {
        if current.is_some() {
            return Err("ready_for_judgment is forbidden while current_tasks.txt has a [NEXT] task".to_string());
        }
        if !all_tasks_done()? {
            return Err("ready_for_judgment is forbidden while TODO tasks remain".to_string());
        }
        let invalid_cells = invalid_product_cells(config, false)?;
        if !invalid_cells.is_empty() {
            return Err(format!(
                "ready_for_judgment rejected; invalid product cells: {}",
                invalid_cells.join(", ")
            ));
        }
        fs::create_dir_all("state").map_err(|err| format!("failed to create state/: {err}"))?;
        let summary = action.summary.clone();
        let marker = json!({
            "declared_by": "ox-creature-self-build-loop",
            "cycle": next_cycle,
            "summary": summary,
            "user_story_to_code_proof": "current_tasks complete and user-story/code materialization path exists",
            "cockpit_proof": "cockpit surface has state, flow, diff, risk, proof, and judgment blocks",
            "validation_proof": "self-build-ready-check validates this marker before Judgment Day",
            "creator_judgment_requested": true
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
        return rejection_cycle(
            config,
            state,
            format!("LLM requested stop instead of completing the task: {}", action.summary),
        );
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
                .unwrap_or_else(|| format!("self-build: sleep cycle {}", state.cycle)),
            should_continue: true,
            ready_for_judgment: false,
            changed_files: vec![STATE_PATH.to_string()],
            delay_seconds: action.delay_seconds.unwrap_or(config.loop_delay_seconds),
        });
    }

    if decision != "continue" {
        return Err(format!("unsupported self-build decision: {decision}"));
    }

    let task = current.ok_or_else(|| {
        "decision=continue is forbidden because current_tasks.txt has no [NEXT] task".to_string()
    })?;
    validate_task_action(&action, &task, config)?;

    let mut backups = Vec::new();
    let mut material_changes = Vec::new();
    for file in &action.files {
        let path = validate_mutation_path(&file.path, config)?;
        let text = mutation_file_text(file)?;
        if text.len() > config.max_bytes_per_file {
            return Err(format!(
                "{} exceeds max_bytes_per_file: {} > {}",
                file.path,
                text.len(),
                config.max_bytes_per_file
            ));
        }
        scan_for_secret_markers(&file.path, &text)?;
        let previous = fs::read_to_string(&path).ok();
        if previous.as_deref() == Some(text.as_str()) {
            continue;
        }
        backups.push(FileBackup {
            path: file.path.clone(),
            previous,
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        fs::write(&path, text)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        material_changes.push(file.path.clone());
    }

    if material_changes.is_empty() {
        rollback_files(&backups)?;
        return rejection_cycle(
            config,
            state,
            format!("task {} rejected: action produced no material file changes", task.id),
        );
    }

    if let Err(err) = run_deterministic_repair_pass(&material_changes) {
        rollback_files(&backups)?;
        return rejection_cycle(
            config,
            state,
            format!("task {} deterministic repair failed after rollback: {err}", task.id),
        );
    }

    if let Err(err) = run_task_validation(&task) {
        rollback_files(&backups)?;
        return rejection_cycle(
            config,
            state,
            format!("task {} validation failed after rollback: {err}", task.id),
        );
    }

    advance_task_ledger(&task.id)?;
    state.cycle = next_cycle;
    state.phase = "self-building".to_string();
    state.status = "task-completed".to_string();
    state.last_summary = format!("Completed task {}: {}", task.id, task.title);
    state.ready_for_judgment = false;
    write_json(STATE_PATH, state)?;

    let mut changed = material_changes;
    changed.push(TASKS_PATH.to_string());
    changed.push(STATE_PATH.to_string());

    Ok(SelfBuildResult {
        status: "task-completed".to_string(),
        cycle: state.cycle,
        summary: state.last_summary.clone(),
        commit_message: action
            .commit_message
            .unwrap_or_else(|| format!("self-build: complete task {}", task.id)),
        should_continue: true,
        ready_for_judgment: false,
        changed_files: changed,
        delay_seconds: action.delay_seconds.unwrap_or(config.loop_delay_seconds),
    })
}

fn validate_task_action(
    action: &LlmMutationAction,
    task: &SelfBuildTask,
    config: &SelfBuildConfig,
) -> Result<(), String> {
    let task_id = action
        .task_id
        .as_deref()
        .ok_or_else(|| "decision=continue requires task_id".to_string())?;
    if task_id != task.id {
        return Err(format!(
            "action task_id mismatch: expected {}, got {}",
            task.id, task_id
        ));
    }
    if action.files.is_empty() {
        return Err("decision=continue requires at least one file".to_string());
    }
    if action.files.len() > config.max_files_per_cycle {
        return Err(format!(
            "mutation touches too many files: {} > {}",
            action.files.len(), config.max_files_per_cycle
        ));
    }
    for file in &action.files {
        if !task.allowed_files.iter().any(|allowed| allowed == &file.path) {
            return Err(format!(
                "{} is not allowed for task {}; allowed files: {}",
                file.path,
                task.id,
                task.allowed_files.join(", ")
            ));
        }
        if file.path == TASKS_PATH || file.path.starts_with("state/") || file.path.starts_with(".github/") {
            return Err(format!(
                "LLM may not mutate ledger/state/workflows directly: {}",
                file.path
            ));
        }
    }
    Ok(())
}

fn mutation_file_text(file: &MutationFile) -> Result<String, String> {
    if let Some(content) = &file.content {
        return Ok(content.clone());
    }
    if let Some(content_base64) = &file.content_base64 {
        let bytes = BASE64_STANDARD
            .decode(content_base64)
            .map_err(|err| format!("failed to decode base64 for {}: {err}", file.path))?;
        return String::from_utf8(bytes)
            .map_err(|err| format!("{} is not valid UTF-8: {err}", file.path));
    }
    Err(format!(
        "{} must include either content or content_base64",
        file.path
    ))
}

fn rejection_cycle(
    config: &SelfBuildConfig,
    state: &mut SelfBuildState,
    reason: String,
) -> Result<SelfBuildResult, String> {
    let next_cycle = state.cycle + 1;
    fs::create_dir_all("state/self-build-rejections")
        .map_err(|err| format!("failed to create rejection directory: {err}"))?;
    let rejection_path = format!("state/self-build-rejections/cycle-{next_cycle:06}.md");
    fs::write(
        &rejection_path,
        format!("# Self-build rejection {next_cycle}\n\n{reason}\n"),
    )
    .map_err(|err| format!("failed to write {rejection_path}: {err}"))?;
    record_experience(&format!("Self-build cycle {next_cycle} rejected. {reason}"))?;

    state.cycle = next_cycle;
    state.phase = "self-building".to_string();
    state.status = "action-rejected".to_string();
    state.last_summary = reason.clone();
    state.ready_for_judgment = false;
    write_json(STATE_PATH, state)?;

    Ok(SelfBuildResult {
        status: "action-rejected".to_string(),
        cycle: state.cycle,
        summary: reason,
        commit_message: format!("self-build: learn from rejected cycle {}", state.cycle),
        should_continue: state.cycle < config.max_cycles,
        ready_for_judgment: false,
        changed_files: vec![
            rejection_path,
            "EXPERIENCE.md".to_string(),
            STATE_PATH.to_string(),
        ],
        delay_seconds: config.loop_delay_seconds,
    })
}

fn rollback_files(backups: &[FileBackup]) -> Result<(), String> {
    for backup in backups.iter().rev() {
        match &backup.previous {
            Some(text) => fs::write(&backup.path, text)
                .map_err(|err| format!("failed to restore {}: {err}", backup.path))?,
            None => {
                let path = Path::new(&backup.path);
                if path.exists() {
                    fs::remove_file(path)
                        .map_err(|err| format!("failed to remove {}: {err}", backup.path))?;
                }
            }
        }
    }
    Ok(())
}

fn run_deterministic_repair_pass(changed_files: &[String]) -> Result<(), String> {
    let touches_rust = changed_files.iter().any(|path| path.ends_with(".rs"));
    if !touches_rust {
        return Ok(());
    }

    let output = Command::new("cargo")
        .args(["fmt", "--all"])
        .output()
        .map_err(|err| format!("failed to run deterministic rustfmt repair: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "deterministic rustfmt repair failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn run_task_validation(task: &SelfBuildTask) -> Result<(), String> {
    if task.validation.is_empty() {
        return Err(format!("task {} has no validation commands", task.id));
    }
    for command in &task.validation {
        run_allowed_validation_command(command)?;
    }
    Ok(())
}

fn run_allowed_validation_command(command: &str) -> Result<(), String> {
    let command = command.trim().trim_end_matches('.').trim();
    let (program, args): (&str, Vec<&str>) = match command {
        "cargo fmt --all --check" => ("cargo", vec!["fmt", "--all", "--check"]),
        "cargo check" => ("cargo", vec!["check"]),
        "cargo test" => ("cargo", vec!["test"]),
        "cargo run -- preflight-check" => ("cargo", vec!["run", "--", "preflight-check"]),
        "cargo run -- git-status --json" => ("cargo", vec!["run", "--", "git-status", "--json"]),
        "cargo run -- creature-init --json" => ("cargo", vec!["run", "--", "creature-init", "--json"]),
        "cargo run -- user-story-intake --demo --json" => {
            ("cargo", vec!["run", "--", "user-story-intake", "--demo", "--json"])
        }
        "cargo run -- intent-candidate --demo --json" => {
            ("cargo", vec!["run", "--", "intent-candidate", "--demo", "--json"])
        }
        "cargo run -- plan-object --demo --json" => {
            ("cargo", vec!["run", "--", "plan-object", "--demo", "--json"])
        }
        "cargo run -- proposal-object --demo --json" => {
            ("cargo", vec!["run", "--", "proposal-object", "--demo", "--json"])
        }
        "cargo run -- patch-artifact --demo --json" => {
            ("cargo", vec!["run", "--", "patch-artifact", "--demo", "--json"])
        }
        "cargo run -- branch-guard --demo --json" => {
            ("cargo", vec!["run", "--", "branch-guard", "--demo", "--json"])
        }
        "cargo run -- rollback-note --demo --json" => {
            ("cargo", vec!["run", "--", "rollback-note", "--demo", "--json"])
        }
        "cargo run -- judgment-report --demo --json" => {
            ("cargo", vec!["run", "--", "judgment-report", "--demo", "--json"])
        }
        "cargo run -- experience-write --demo --json" => {
            ("cargo", vec!["run", "--", "experience-write", "--demo", "--json"])
        }
        "cargo run -- cockpit-demo --json" => ("cargo", vec!["run", "--", "cockpit-demo", "--json"]),
        "cargo run -- first-meeting-demo --json" => {
            ("cargo", vec!["run", "--", "first-meeting-demo", "--json"])
        }
        other => return Err(format!("validation command is not allowlisted: {other}")),
    };
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run validation command `{command}`: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "validation command failed `{command}`\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
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
    for file in &action.files {
        if file.path.trim().is_empty() {
            return Err("mutation file path is empty".to_string());
        }
        if file.content.is_none() && file.content_base64.is_none() {
            return Err(format!(
                "{} must include either content or content_base64",
                file.path
            ));
        }
        if file.content.is_some() && file.content_base64.is_some() {
            return Err(format!(
                "{} must not include both content and content_base64",
                file.path
            ));
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
    if path == TASKS_PATH || path == STATE_PATH || path == RESULT_PATH {
        return Err(format!("self-build action may not mutate runtime-owned file: {path}"));
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

fn current_task() -> Result<Option<SelfBuildTask>, String> {
    Ok(parse_tasks()?.into_iter().find(|task| task.status == "NEXT"))
}

fn all_tasks_done() -> Result<bool, String> {
    Ok(parse_tasks()?
        .iter()
        .all(|task| task.status == "DONE" || task.status == "SKIP"))
}

fn parse_tasks() -> Result<Vec<SelfBuildTask>, String> {
    let text = read_optional(TASKS_PATH);
    let lines: Vec<&str> = text.lines().collect();
    let mut tasks = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i].trim();
        let Some((status, rest)) = parse_task_header(line) else {
            i += 1;
            continue;
        };
        let mut parts = rest.splitn(2, ' ');
        let id = parts.next().unwrap_or_default().trim().to_string();
        let title = parts.next().unwrap_or_default().trim().to_string();
        let mut goal = String::new();
        let mut allowed_files = Vec::new();
        let mut validation = Vec::new();
        i += 1;
        while i < lines.len() && parse_task_header(lines[i].trim()).is_none() {
            let current = lines[i].trim();
            if let Some(value) = current.strip_prefix("Goal:") {
                goal = strip_markdown(value.trim());
            } else if let Some(value) = current.strip_prefix("Allowed files:") {
                allowed_files = split_csv_like(value);
            } else if let Some(value) = current.strip_prefix("Validation:") {
                validation = split_validation(value);
            }
            i += 1;
        }
        if !id.is_empty() {
            tasks.push(SelfBuildTask {
                id,
                title,
                status,
                goal,
                allowed_files,
                validation,
            });
        }
    }
    Ok(tasks)
}

fn parse_task_header(line: &str) -> Option<(String, String)> {
    for status in ["DONE", "NEXT", "TODO", "SKIP"] {
        let prefix = format!("[{status}] ");
        if let Some(rest) = line.strip_prefix(&prefix) {
            return Some((status.to_string(), rest.to_string()));
        }
    }
    None
}

fn split_csv_like(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_end_matches('.')
        .split(',')
        .map(strip_markdown)
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn split_validation(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_end_matches('.')
        .split("&&")
        .map(strip_markdown)
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn strip_markdown(value: &str) -> String {
    value.replace('`', "").trim().to_string()
}

fn advance_task_ledger(completed_task_id: &str) -> Result<(), String> {
    let text = read_optional(TASKS_PATH);
    let mut advanced_next = false;
    let mut completed = false;
    let mut output = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !completed && trimmed.starts_with(&format!("[NEXT] {completed_task_id} ")) {
            output.push(line.replacen("[NEXT]", "[DONE]", 1));
            completed = true;
        } else if completed && !advanced_next && trimmed.starts_with("[TODO] ") {
            output.push(line.replacen("[TODO]", "[NEXT]", 1));
            advanced_next = true;
        } else {
            output.push(line.to_string());
        }
    }
    if !completed {
        return Err(format!(
            "could not advance task ledger; NEXT task {completed_task_id} not found"
        ));
    }
    fs::write(TASKS_PATH, format!("{}\n", output.join("\n")))
        .map_err(|err| format!("failed to write {TASKS_PATH}: {err}"))
}

fn invalid_product_cells(config: &SelfBuildConfig, include_marker: bool) -> Result<Vec<String>, String> {
    let mut invalid = Vec::new();
    for cell in target_cells(config) {
        if !include_marker && cell.id == "judgment.readiness_marker" {
            continue;
        }
        if cell.status != "valid" {
            invalid.push(format!("{}={}", cell.id, cell.proof));
        }
    }
    Ok(invalid)
}

fn target_cells(config: &SelfBuildConfig) -> Vec<TargetCell> {
    vec![
        proof_cell(
            "runtime.self_build_engine",
            "Runtime can produce task-driven context, accept LLM patch JSON, validate it, rollback bad changes, mutate files, advance tasks, and update state.",
            "src/self_build.rs",
            &["current_task", "advance_task_ledger", "rollback_files", "run_task_validation"],
        ),
        proof_cell(
            "runtime.self_build_rate",
            "Self-build loop delay is configurable in seconds and defaults to zero.",
            CONFIG_PATH,
            &["loop_delay_seconds"],
        ),
        proof_cell(
            "contract.llm_next_action",
            "LLM output contract exists and constrains mutations to small validated task-scoped patches.",
            "contracts/llm-next-action.schema.json",
            &["task_id", "allowed", "files", "content"],
        ),
        proof_cell(
            "github.self_build_loop",
            "GitHub Actions can run the self-build cycle, commit, push, and dispatch continuation.",
            ".github/workflows/self-loop.yml",
            &["workflow_dispatch", "contents: write", "actions: write"],
        ),
        proof_cell(
            "user_story.intake_contract",
            "A user story has a typed contract before it can become code.",
            "contracts/user-story.schema.json",
            &["acceptance_criteria", "actor", "goal", "reason", "proof_expected"],
        ),
        proof_cell(
            "materialization.patch_contract",
            "Code materialization is represented as a bounded patch artifact before execution.",
            "contracts/materialized-patch.schema.json",
            &["files", "hashes", "validation_commands", "rollback", "proof"],
        ),
        proof_cell(
            "cockpit.surface",
            "Cockpit has a visible surface that can show creature state, flow, diffs, risks, proof, and Judgment readiness.",
            "apps/cockpit/README.md",
            &["state", "flow", "diff", "risk", "proof", "Judgment"],
        ),
        proof_cell(
            "judgment.readiness_marker",
            "The creature can stop the self-build loop and request Judgment Day only after product proof exists.",
            &config.judgment_ready_marker,
            &[
                "user_story_to_code_proof",
                "cockpit_proof",
                "validation_proof",
                "creator_judgment_requested",
            ],
        ),
    ]
}

fn proof_cell(id: &str, purpose: &str, proof_path: &str, required_markers: &[&str]) -> TargetCell {
    let path = Path::new(proof_path);
    let proof = if !path.is_file() {
        "missing".to_string()
    } else {
        let text = read_optional(proof_path);
        let missing: Vec<&str> = required_markers
            .iter()
            .copied()
            .filter(|marker| !text.contains(marker))
            .collect();
        if missing.is_empty() {
            "content markers valid".to_string()
        } else {
            format!("weak; missing markers: {}", missing.join(", "))
        }
    };
    let status = if proof == "content markers valid" {
        "valid"
    } else {
        "invalid"
    };
    TargetCell {
        id: id.to_string(),
        purpose: purpose.to_string(),
        proof_path: proof_path.to_string(),
        status: status.to_string(),
        proof,
    }
}

fn load_config() -> Result<SelfBuildConfig, String> {
    let path = Path::new(CONFIG_PATH);
    if !path.is_file() {
        return Ok(SelfBuildConfig::default());
    }
    let text =
        fs::read_to_string(path).map_err(|err| format!("failed to read {CONFIG_PATH}: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid {CONFIG_PATH}: {err}"))
}

fn load_state() -> Result<SelfBuildState, String> {
    let path = Path::new(STATE_PATH);
    if !path.is_file() {
        return Ok(SelfBuildState::default());
    }
    let text =
        fs::read_to_string(path).map_err(|err| format!("failed to read {STATE_PATH}: {err}"))?;
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
        let mut end = max;
        while !text.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}\n...[truncated]", &text[..end])
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
                    let begin =
                        start.ok_or_else(|| "internal JSON extraction error".to_string())?;
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
