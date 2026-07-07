# EXPERIENCE

No real failures recorded yet.

This file must record serious failures after Judgment Day or local validation failure.

## Self-build experience

Self-build cycle 29 rejected. task 006 validation failed after rollback: validation command failed `cargo fmt --all --check`
stdout:
Diff in /home/runner/work/ox-creature/ox-creature/src/main.rs:17:
         "creature-init" => {
             let json_flag = args.iter().any(|a| a == "--json");
             let workspace_dir = Path::new(".creature");
-            
+
             let status = if workspace_dir.exists() {
                 "validated"
             } else {
Diff in /home/runner/work/ox-creature/ox-creature/src/main.rs:36:
                 });
                 println!("{}", serde_json::to_string_pretty(&output).unwrap());
             } else {
-                println!("Creature workspace {} at {}", status, workspace_dir.display());
+                println!(
+                    "Creature workspace {} at {}",
+                    status,
+                    workspace_dir.display()
+                );
             }
         }
         _ => {

stderr:


## Self-build experience

Self-build cycle 31 rejected. LLM action rejected before commit: src/main.rs mutation would break protected runtime command surface; missing markers: mod self_build;, seed-check, flow-check, contract-check, llm-config-check, self-build-rate-seconds, self-build-context, llm-action-from-response, self-build-step, self-build-ready-check, self_build::self_build_rate_seconds(), self_build::self_build_context(&args), self_build::llm_action_from_response(&args), self_build::self_build_step(&args), self_build::self_build_ready_check()
