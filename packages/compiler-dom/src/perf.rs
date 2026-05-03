//! Compile-time perf report (S71 Day 2).
//!
//! Off by default. Set `CompileOptions.collect_perf_report = true` to populate
//! `CompileResult.perf_report`. The `voce compile --perf-report <path>` CLI
//! flag writes the report as a JSON sidecar; tests verify the shape.
//!
//! Phases timed:
//!   ingest    JSON → CompilerIr (parse + lower into typed nodes)
//!   emit      CompilerIr → HtmlOutput (the actual HTML string buildup)
//!   minify    optional whitespace collapse pass (when CompileOptions.minify)
//!
//! Outer-process work — reading the file, running the validator, writing the
//! output — is timed by the CLI itself and merged into the report under the
//! same `phases` map (see voce-validator's cmd_compile).

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// One phase's timing in a PerfReport. Serialized as `{ name, us }` so
/// consumers can index into a phase by name without tuple-position guessing.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PhaseTime {
    pub name: String,
    /// Wall time in microseconds.
    pub us: u128,
}

/// Single-compile perf report. Fields match the S71 spec format.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerfReport {
    /// Bytes of input JSON consumed by `compile()`.
    pub input_bytes: usize,
    /// Bytes of HTML output produced.
    pub output_bytes: usize,
    /// Total `compile()` wall time, microseconds. Sum of phases plus a small
    /// overhead for the bookkeeping itself.
    pub total_us: u128,
    /// Per-phase microseconds. Insertion-ordered; iterating yields phases in
    /// the order they ran.
    pub phases: Vec<PhaseTime>,
    /// Number of nodes in the lowered CompilerIr.
    pub node_count: usize,
    /// ISO-8601 timestamp at the moment compile started. e.g. "2026-05-03T12:45:00Z".
    pub compiled_at: String,
}

impl PerfReport {
    /// Helper for the CLI / consumers that want to attach an outer-process
    /// timing (e.g. validate, file read, file write) to the report.
    pub fn add_phase(&mut self, name: impl Into<String>, duration: Duration) {
        self.phases.push(PhaseTime {
            name: name.into(),
            us: duration.as_micros(),
        });
    }

    /// Convert to a pretty-printed JSON string. Useful for the `--perf-report`
    /// sidecar; rendering errors are unreachable for this shape.
    pub fn to_json_pretty(&self) -> String {
        // serde_json::to_string_pretty cannot fail on this shape; unwrap is safe.
        #[allow(clippy::expect_used)]
        serde_json::to_string_pretty(self).expect("PerfReport always serializes")
    }
}

/// In-progress collection helper. Wraps a `SystemTime` start + a `Vec` of
/// (name, started_at) pairs so phase timings can be `start_phase`/`end_phase`'d
/// without nesting closures.
pub(crate) struct PerfCollector {
    process_started: std::time::Instant,
    started_at_iso: String,
    phases: Vec<PhaseTime>,
    current_phase: Option<(String, std::time::Instant)>,
}

impl PerfCollector {
    pub(crate) fn start() -> Self {
        Self {
            process_started: std::time::Instant::now(),
            started_at_iso: now_iso8601(),
            phases: Vec::new(),
            current_phase: None,
        }
    }

    pub(crate) fn start_phase(&mut self, name: &str) {
        // Defensive: if a previous phase wasn't ended, end it implicitly so
        // the timing isn't lost (the bug surfaces in tests but we still want
        // a working report in production).
        if let Some((prev_name, prev_start)) = self.current_phase.take() {
            self.phases.push(PhaseTime {
                name: prev_name,
                us: prev_start.elapsed().as_micros(),
            });
        }
        self.current_phase = Some((name.to_string(), std::time::Instant::now()));
    }

    pub(crate) fn end_phase(&mut self) {
        if let Some((name, start)) = self.current_phase.take() {
            self.phases.push(PhaseTime {
                name,
                us: start.elapsed().as_micros(),
            });
        }
    }

    pub(crate) fn finish(
        mut self,
        input_bytes: usize,
        output_bytes: usize,
        node_count: usize,
    ) -> PerfReport {
        // Make sure no phase is left dangling.
        self.end_phase();
        PerfReport {
            input_bytes,
            output_bytes,
            total_us: self.process_started.elapsed().as_micros(),
            phases: self.phases,
            node_count,
            compiled_at: self.started_at_iso,
        }
    }
}

/// Format the current wall time as ISO-8601 in UTC: `YYYY-MM-DDTHH:MM:SSZ`.
/// Inline implementation — adding a `chrono` / `time` dep just for one
/// timestamp would be overkill and would inflate the WASM bundle.
fn now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_iso8601_utc(secs)
}

/// Convert a Unix epoch seconds count into ISO-8601 UTC. Pure function; the
/// caller picks the source. Handles dates 1970..9999. Days-since-epoch math
/// is Howard Hinnant's algorithm — same one chrono uses internally — but
/// stripped to integer ops.
pub(crate) fn format_iso8601_utc(unix_seconds: i64) -> String {
    let secs_in_day: i64 = 86_400;
    let days = unix_seconds.div_euclid(secs_in_day);
    let time_of_day = unix_seconds.rem_euclid(secs_in_day);

    let (y, m, d) = days_to_civil(days);
    let h = time_of_day / 3600;
    let mi = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Howard Hinnant's date algorithm. Days since 1970-01-01 → (year, month, day).
/// month is 1-12, day is 1-31.
fn days_to_civil(days: i64) -> (i64, u32, u32) {
    // Shift epoch from 1970-01-01 to 0000-03-01.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_format_handles_known_dates() {
        // Unix epoch.
        assert_eq!(format_iso8601_utc(0), "1970-01-01T00:00:00Z");
        // Y2K.
        assert_eq!(format_iso8601_utc(946_684_800), "2000-01-01T00:00:00Z");
        // S71 launch reference (1_780_000_000 ≈ mid-2026).
        assert_eq!(format_iso8601_utc(1_780_000_000), "2026-05-28T20:26:40Z");
    }

    #[test]
    fn iso_format_round_trips_within_a_day() {
        // 1234567 seconds = 14 days + something. Pick a stable reference.
        let s = format_iso8601_utc(1_234_567);
        assert_eq!(s, "1970-01-15T06:56:07Z");
    }

    #[test]
    fn perf_collector_emits_ordered_phases() {
        let mut p = PerfCollector::start();
        p.start_phase("first");
        std::thread::sleep(Duration::from_micros(50));
        p.start_phase("second");
        std::thread::sleep(Duration::from_micros(50));
        let report = p.finish(100, 200, 5);
        assert_eq!(report.phases.len(), 2);
        assert_eq!(report.phases[0].name, "first");
        assert_eq!(report.phases[1].name, "second");
        assert!(report.phases[0].us >= 1, "phase 0 should record some time");
        assert_eq!(report.input_bytes, 100);
        assert_eq!(report.output_bytes, 200);
        assert_eq!(report.node_count, 5);
    }

    #[test]
    fn report_serializes_with_expected_keys() {
        let report = PerfReport {
            input_bytes: 100,
            output_bytes: 200,
            total_us: 1234,
            phases: vec![
                PhaseTime {
                    name: "ingest".into(),
                    us: 500,
                },
                PhaseTime {
                    name: "emit".into(),
                    us: 700,
                },
            ],
            node_count: 5,
            compiled_at: "2026-05-03T12:00:00Z".into(),
        };
        let json = report.to_json_pretty();
        assert!(json.contains("\"input_bytes\""));
        assert!(json.contains("\"output_bytes\""));
        assert!(json.contains("\"total_us\""));
        assert!(json.contains("\"phases\""));
        assert!(json.contains("\"node_count\""));
        assert!(json.contains("\"compiled_at\""));
        // Phases serialize as { name, us } objects, not tuples.
        assert!(json.contains("\"name\": \"ingest\""));
        assert!(json.contains("\"us\": 500"));
        // Phase order preserved.
        let i_idx = json.find("ingest").unwrap();
        let e_idx = json.find("emit").unwrap();
        assert!(i_idx < e_idx);
    }

    #[test]
    fn add_phase_appends_at_end() {
        let mut report = PerfReport {
            input_bytes: 0,
            output_bytes: 0,
            total_us: 0,
            phases: vec![PhaseTime {
                name: "ingest".into(),
                us: 100,
            }],
            node_count: 0,
            compiled_at: String::new(),
        };
        report.add_phase("validate", Duration::from_micros(250));
        let last = report.phases.last().unwrap();
        assert_eq!(last.name, "validate");
        assert_eq!(last.us, 250);
    }
}
