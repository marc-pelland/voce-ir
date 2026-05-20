//! S79 A2 — `voce doctor`: toolchain + project (`.voce/`) health.
//!
//! Project-level analog of `voce fix`. Each check has a **stable ID**
//! (part of the agent contract, same versioning rules as diagnostic
//! codes) and emits structured status + hint + docs URL so an agent
//! can drive remediation without re-parsing prose.
//!
//! Slice 1 covers toolchain (`flatc`) and `.voce/` integrity against
//! the canonical layout in `.voce/SCHEMA.md`. Per-IR validation
//! (`*.voce.json` files in CWD) is A2 Slice 2.

use std::path::Path;
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    /// Not applicable in this context (e.g. `.voce/` checks when the
    /// project doesn't use `.voce/` at all).
    Skip,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Check {
    /// Stable, contract-versioned ID. Format: `DOC-<DOMAIN>-NNN`.
    pub id: &'static str,
    pub title: &'static str,
    pub status: CheckStatus,
    /// Concrete observation (e.g. "flatc 24.3.25 found at /usr/local/bin/flatc").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Actionable next step when status is warn/fail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'static str>,
    pub docs_url: String,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct DoctorReport {
    pub contract_version: &'static str,
    /// Resolved CWD checks were run against.
    pub project_root: String,
    /// `true` when no `fail` (and, under `--strict`, no `warn`) checks.
    pub ok: bool,
    pub strict: bool,
    pub summary: ReportSummary,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, Default, Serialize, schemars::JsonSchema)]
pub struct ReportSummary {
    pub pass: usize,
    pub warn: usize,
    pub fail: usize,
    pub skip: usize,
}

fn docs(id: &str) -> String {
    format!("https://voce-ir.xyz/docs/doctor/{id}")
}

/// Run the full doctor suite against `project_root`. `strict` promotes
/// `warn` to a non-zero exit when the report drives a process exit.
pub fn run(project_root: &Path, strict: bool) -> DoctorReport {
    let mut checks = Vec::new();
    checks.extend(toolchain_checks());
    checks.extend(voce_dir_checks(project_root));

    let mut summary = ReportSummary::default();
    for c in &checks {
        match c.status {
            CheckStatus::Pass => summary.pass += 1,
            CheckStatus::Warn => summary.warn += 1,
            CheckStatus::Fail => summary.fail += 1,
            CheckStatus::Skip => summary.skip += 1,
        }
    }

    let ok = summary.fail == 0 && (!strict || summary.warn == 0);

    DoctorReport {
        contract_version: crate::skills::CONTRACT_VERSION,
        project_root: project_root.display().to_string(),
        ok,
        strict,
        summary,
        checks,
    }
}

// ─── Toolchain checks ───────────────────────────────────────────

fn toolchain_checks() -> Vec<Check> {
    vec![flatc_check(), contract_version_check()]
}

fn flatc_check() -> Check {
    const ID: &str = "DOC-TOOLCHAIN-001";
    let detail = Command::new("flatc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        });

    match detail {
        Some(v) if !v.is_empty() => Check {
            id: ID,
            title: "flatc available",
            status: CheckStatus::Pass,
            detail: Some(v),
            hint: None,
            docs_url: docs(ID),
        },
        _ => Check {
            id: ID,
            title: "flatc available",
            status: CheckStatus::Warn,
            detail: Some("flatc not found on PATH".into()),
            hint: Some(
                "Install flatc (the FlatBuffers compiler) — used by `voce json2bin` / `bin2json` \
                 and to regenerate schema bindings. macOS: `brew install flatbuffers`.",
            ),
            docs_url: docs(ID),
        },
    }
}

fn contract_version_check() -> Check {
    const ID: &str = "DOC-TOOLCHAIN-002";
    Check {
        id: ID,
        title: "Agent contract version pinned",
        status: CheckStatus::Pass,
        detail: Some(format!(
            "contract v{} (voce v{})",
            crate::skills::CONTRACT_VERSION,
            env!("CARGO_PKG_VERSION")
        )),
        hint: None,
        docs_url: docs(ID),
    }
}

// ─── .voce/ project checks (per .voce/SCHEMA.md) ─────────────────

fn voce_dir_checks(project_root: &Path) -> Vec<Check> {
    let voce_dir = project_root.join(".voce");
    if !voce_dir.exists() {
        return vec![Check {
            id: "DOC-VOCE-001",
            title: ".voce/ project memory present",
            status: CheckStatus::Skip,
            detail: Some(format!(
                "No .voce/ at {} — project memory not in use",
                project_root.display()
            )),
            hint: Some("Optional. `.voce/brief.md` enables drift detection and decision logging."),
            docs_url: docs("DOC-VOCE-001"),
        }];
    }

    vec![
        voce_dir_present(&voce_dir),
        brief_present(&voce_dir),
        jsonl_parseable(
            &voce_dir,
            "DOC-VOCE-003",
            "decisions.jsonl parseable",
            "decisions.jsonl",
        ),
        jsonl_parseable(
            &voce_dir,
            "DOC-VOCE-004",
            "drift-warnings.jsonl parseable",
            "drift-warnings.jsonl",
        ),
    ]
}

fn voce_dir_present(voce_dir: &Path) -> Check {
    const ID: &str = "DOC-VOCE-001";
    Check {
        id: ID,
        title: ".voce/ project memory present",
        status: CheckStatus::Pass,
        detail: Some(voce_dir.display().to_string()),
        hint: None,
        docs_url: docs(ID),
    }
}

fn brief_present(voce_dir: &Path) -> Check {
    const ID: &str = "DOC-VOCE-002";
    let brief = voce_dir.join("brief.md");
    if brief.exists() {
        Check {
            id: ID,
            title: ".voce/brief.md present",
            status: CheckStatus::Pass,
            detail: Some(brief.display().to_string()),
            hint: None,
            docs_url: docs(ID),
        }
    } else {
        Check {
            id: ID,
            title: ".voce/brief.md present",
            status: CheckStatus::Warn,
            detail: Some(format!("Missing {}", brief.display())),
            hint: Some(
                "Without a project brief, drift detection is disabled. \
                 Create `.voce/brief.md` (the project north star) or use \
                 the `voce_brief_set` MCP tool.",
            ),
            docs_url: docs(ID),
        }
    }
}

fn jsonl_parseable(
    voce_dir: &Path,
    id: &'static str,
    title: &'static str,
    filename: &str,
) -> Check {
    let path = voce_dir.join(filename);
    if !path.exists() {
        // Absence is fine — the log just hasn't been written yet.
        return Check {
            id,
            title,
            status: CheckStatus::Pass,
            detail: Some(format!("{} not yet written", path.display())),
            hint: None,
            docs_url: docs(id),
        };
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            return Check {
                id,
                title,
                status: CheckStatus::Fail,
                detail: Some(format!("read {}: {e}", path.display())),
                hint: Some("Check file permissions; the file should be readable to anyone who can run `voce`."),
                docs_url: docs(id),
            };
        }
    };
    let mut bad_lines: Vec<usize> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if serde_json::from_str::<serde_json::Value>(trimmed).is_err() {
            bad_lines.push(i + 1);
        }
    }
    if bad_lines.is_empty() {
        Check {
            id,
            title,
            status: CheckStatus::Pass,
            detail: Some(format!("{} entries parseable", text.lines().count())),
            hint: None,
            docs_url: docs(id),
        }
    } else {
        Check {
            id,
            title,
            status: CheckStatus::Fail,
            detail: Some(format!(
                "{} unparseable line(s) at {:?}",
                bad_lines.len(),
                bad_lines
            )),
            hint: Some(
                "Per .voce/SCHEMA.md, JSONL files are append-only and one JSON object \
                 per line. A bad line indicates an out-of-band edit; remove it (the log \
                 is recoverable because supersession is expressed via new entries).",
            ),
            docs_url: docs(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmpdir() -> PathBuf {
        // Nano timestamps can collide between parallel tests on fast
        // machines; pair with a process-unique atomic counter so each
        // caller gets a fresh, isolated directory.
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "voce-doctor-test-{nanos}-{}-{n}",
            std::process::id()
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn report_carries_contract_version_and_resolved_root() {
        let root = tmpdir();
        let r = run(&root, false);
        assert_eq!(r.contract_version, crate::skills::CONTRACT_VERSION);
        assert_eq!(r.project_root, root.display().to_string());
    }

    #[test]
    fn no_voce_dir_yields_skip_not_fail() {
        let root = tmpdir();
        let r = run(&root, false);
        let voce = r.checks.iter().find(|c| c.id == "DOC-VOCE-001").unwrap();
        assert_eq!(voce.status, CheckStatus::Skip);
        assert!(r.ok, "no .voce/ is not a failure condition");
    }

    #[test]
    fn missing_brief_warns_but_does_not_fail() {
        let root = tmpdir();
        fs::create_dir_all(root.join(".voce")).unwrap();
        let r = run(&root, false);
        let brief = r.checks.iter().find(|c| c.id == "DOC-VOCE-002").unwrap();
        assert_eq!(brief.status, CheckStatus::Warn);
        assert!(r.ok, "warn alone does not flip ok in non-strict mode");
        assert!(brief.hint.is_some());
    }

    #[test]
    fn strict_mode_promotes_warn_to_not_ok() {
        let root = tmpdir();
        fs::create_dir_all(root.join(".voce")).unwrap();
        let r = run(&root, true);
        assert!(!r.ok, "strict mode must treat warnings as not-ok");
    }

    #[test]
    fn malformed_jsonl_fails_decisively() {
        let root = tmpdir();
        fs::create_dir_all(root.join(".voce")).unwrap();
        fs::write(
            root.join(".voce/decisions.jsonl"),
            "{\"id\":\"a\",\"timestamp\":\"x\"}\nnot json\n{}\n",
        )
        .unwrap();
        let r = run(&root, false);
        let decisions = r.checks.iter().find(|c| c.id == "DOC-VOCE-003").unwrap();
        assert_eq!(decisions.status, CheckStatus::Fail);
        assert!(!r.ok, "any fail flips ok");
    }

    #[test]
    fn well_formed_voce_dir_passes() {
        let root = tmpdir();
        let voce = root.join(".voce");
        fs::create_dir_all(&voce).unwrap();
        fs::write(voce.join("brief.md"), "# Project north star\n").unwrap();
        fs::write(
            voce.join("decisions.jsonl"),
            "{\"id\":\"d1\",\"timestamp\":\"2026-05-20T00:00:00Z\",\"summary\":\"x\",\"rationale\":\"y\"}\n",
        )
        .unwrap();
        let r = run(&root, false);
        for c in &r.checks {
            // Toolchain `flatc` may or may not be installed in CI; both
            // pass and warn are acceptable for that one check.
            if c.id == "DOC-TOOLCHAIN-001" {
                assert!(matches!(c.status, CheckStatus::Pass | CheckStatus::Warn));
                continue;
            }
            assert!(
                matches!(c.status, CheckStatus::Pass),
                "{} should pass, got {:?} ({:?})",
                c.id,
                c.status,
                c.detail
            );
        }
    }
}
