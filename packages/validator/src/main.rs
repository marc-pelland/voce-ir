use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

/// Voce IR — AI-native UI intermediate representation toolchain.
///
/// The code is gone. The experience remains.
#[derive(Parser)]
#[command(name = "voce")]
#[command(version)]
#[command(about = "Voce IR toolchain — validate, compile, and deploy AI-generated interfaces")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose debug output
    #[arg(long, global = true)]
    verbose: bool,

    /// Output errors as JSON (for machine consumption)
    #[arg(long, global = true)]
    json_errors: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Terminal,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an IR file against all quality rules
    Validate {
        /// Path to the IR file (.voce.json). Optional when --list-passes is used.
        file: Option<PathBuf>,

        /// Output format
        #[arg(long, default_value = "terminal")]
        format: OutputFormat,

        /// Treat warnings as errors
        #[arg(long)]
        warn_as_error: bool,

        /// Include per-pass timing and outcome in JSON output (requires --format json)
        #[arg(long)]
        verbose_passes: bool,

        /// Print the canonical list of validation passes (in execution order) and exit
        #[arg(long, conflicts_with_all = ["format", "warn_as_error", "verbose_passes"])]
        list_passes: bool,

        /// Print the catalogue of diagnostic codes (with pass + summary) and exit
        #[arg(long, conflicts_with_all = ["format", "warn_as_error", "verbose_passes", "list_passes"])]
        list_codes: bool,
    },

    /// Inspect an IR file (human-readable summary, not code)
    Inspect {
        /// Path to the IR file
        file: PathBuf,
    },

    /// Convert JSON canonical format to binary FlatBuffer
    Json2bin {
        /// Input JSON file
        input: PathBuf,

        /// Output binary file (defaults to input with .voce extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert binary FlatBuffer to JSON canonical format
    Bin2json {
        /// Input binary file
        input: PathBuf,

        /// Output JSON file (defaults to input with .voce.json extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compile an IR file to HTML
    Compile {
        /// Path to the IR file (.voce.json)
        file: PathBuf,

        /// Output HTML file (defaults to dist/index.html)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include data-voce-id attributes for debugging
        #[arg(long)]
        debug: bool,

        /// Skip font processing (no @font-face, no preloads)
        #[arg(long)]
        skip_fonts: bool,

        /// Minify the HTML output
        #[arg(long)]
        minify: bool,

        /// Disable compilation cache
        #[arg(long)]
        no_cache: bool,

        /// Write a compile-time perf report (JSON sidecar) to this path (S71 Day 2).
        #[arg(long, value_name = "PATH")]
        perf_report: Option<PathBuf>,

        /// Print the cache outcome (HIT / MISS / SKIPPED) for this invocation
        /// and append a structured line to .voce/perf-log.jsonl (S71 Day 5).
        #[arg(long)]
        report_cache: bool,
    },

    /// Generate a compilation quality report
    Report {
        /// Path to the IR file
        file: PathBuf,

        /// Output format
        #[arg(long, default_value = "terminal")]
        format: OutputFormat,
    },

    /// Generate an application manifest from IR
    Manifest {
        /// Path to the IR file
        file: PathBuf,
    },

    /// Compile and preview in the browser
    Preview {
        /// Path to the IR file
        file: PathBuf,
    },

    /// Generate IR from a natural language description (requires AI bridge)
    Generate {
        /// What to build (natural language description)
        prompt: String,

        /// Output IR file (defaults to generated.voce.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Apply auto-fix proposals to an IR file (S67 Day 5)
    Fix {
        /// Path to the IR file (.voce.json)
        file: PathBuf,

        /// Write changes back to the file. Without this flag, runs in
        /// preview mode and prints what would change.
        #[arg(long)]
        apply: bool,

        /// Highest confidence level to apply. Patches with higher confidence
        /// (i.e. less safe) are listed but skipped.
        #[arg(long, default_value = "safe", value_parser = ["safe", "suggested", "risky"])]
        confidence: String,
    },

    /// Deploy compiled output to a hosting platform
    Deploy {
        /// Path to the IR file (.voce.json)
        file: PathBuf,

        /// Deployment adapter: static, vercel, cloudflare, netlify
        #[arg(long)]
        adapter: Option<String>,

        /// Preview the bundle without deploying
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("voce: verbose mode enabled");
    }

    let json_errors = cli.json_errors;
    let exit_code = match run(cli) {
        Ok(code) => code,
        Err(e) => {
            if json_errors {
                let report = serde_json::json!({
                    "errors": [{ "code": "INTERNAL", "message": format!("{e:#}"), "suggestion": "Check the input file and try again" }]
                });
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                );
            } else {
                eprintln!("voce: {e:#}");
            }
            voce_schema::errors::exit_codes::INTERNAL_ERROR
        }
    };

    process::exit(exit_code);
}

fn run(cli: Cli) -> Result<i32> {
    match cli.command {
        Commands::Validate {
            file,
            format,
            warn_as_error,
            verbose_passes,
            list_passes,
            list_codes,
        } => cmd_validate(
            file.as_ref(),
            &format,
            warn_as_error,
            verbose_passes,
            list_passes,
            list_codes,
        ),
        Commands::Inspect { file } => cmd_inspect(&file),
        Commands::Json2bin { input, output } => cmd_json2bin(&input, output.as_deref()),
        Commands::Bin2json { input, output } => cmd_bin2json(&input, output.as_deref()),
        Commands::Compile {
            file,
            output,
            debug,
            skip_fonts,
            minify,
            no_cache,
            perf_report,
            report_cache,
        } => cmd_compile(
            &file,
            output.as_deref(),
            debug,
            skip_fonts,
            minify,
            no_cache,
            perf_report.as_deref(),
            report_cache,
        ),
        Commands::Report { file, format } => cmd_report(&file, &format),
        Commands::Manifest { file } => cmd_manifest(&file),
        Commands::Preview { file } => cmd_preview(&file),
        Commands::Generate { prompt, output } => cmd_generate(&prompt, output.as_deref()),
        Commands::Fix {
            file,
            apply,
            confidence,
        } => cmd_fix(&file, apply, &confidence),
        Commands::Deploy {
            file,
            adapter,
            dry_run,
        } => cmd_deploy(&file, adapter.as_deref(), dry_run),
    }
}

fn cmd_validate(
    file: Option<&PathBuf>,
    format: &OutputFormat,
    warn_as_error: bool,
    verbose_passes: bool,
    list_passes: bool,
    list_codes: bool,
) -> Result<i32> {
    if list_passes {
        let names: Vec<&'static str> = voce_validator::passes::all_passes()
            .iter()
            .map(|p| p.name())
            .collect();
        let out = serde_json::json!({ "passes": names });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(0);
    }

    if list_codes {
        let mut entries: Vec<serde_json::Value> = Vec::new();
        for pass in voce_validator::passes::all_passes() {
            for meta in pass.codes() {
                entries.push(serde_json::json!({
                    "code": meta.code,
                    "pass": pass.name(),
                    "summary": meta.summary,
                    "hint": meta.hint,
                    "fix_confidence": meta.fix_confidence.map(|c| c.to_string()),
                    "docs_url": voce_validator::formatter::docs_url(meta.code),
                }));
            }
        }
        let out = serde_json::json!({ "codes": entries });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(0);
    }

    let file = file.ok_or_else(|| {
        anyhow::anyhow!(
            "validate: missing IR file path (or use --list-passes / --list-codes to enumerate)"
        )
    })?;

    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    // Discover .voce/validator.toml from the IR's directory upward.
    let config_dir = file.parent().unwrap_or(std::path::Path::new("."));
    let config = voce_validator::ValidatorConfig::load_from_dir(config_dir);
    let result =
        voce_validator::validate_with_config(&json, &config).map_err(|e| anyhow::anyhow!("{e}"))?;

    let file_str = file.display().to_string();

    match format {
        OutputFormat::Terminal => {
            if verbose_passes {
                eprintln!("voce validate: --verbose-passes only affects --format json; ignoring");
            }
            voce_validator::formatter::print_terminal(&file_str, &result);
        }
        OutputFormat::Json => {
            if verbose_passes {
                voce_validator::formatter::print_json_verbose(&file_str, &result)
                    .map_err(|e| anyhow::anyhow!("JSON output error: {e}"))?;
            } else {
                voce_validator::formatter::print_json(&file_str, &result)
                    .map_err(|e| anyhow::anyhow!("JSON output error: {e}"))?;
            }
        }
    }

    let has_fatal = result.has_errors() || (warn_as_error && result.warning_count() > 0);
    Ok(if has_fatal { 1 } else { 0 })
}

fn cmd_inspect(file: &PathBuf) -> Result<i32> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    let ir: voce_validator::ir::VoceIr = serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse IR from {}", file.display()))?;

    let summary = voce_validator::inspect::summarize(&ir);
    voce_validator::inspect::print_summary(&file.display().to_string(), &summary);

    Ok(0)
}

fn cmd_json2bin(input: &PathBuf, output: Option<&std::path::Path>) -> Result<i32> {
    let out_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("voce"));

    // Use flatc for JSON → binary conversion
    let schema_dir = find_schema_dir()?;
    let status = std::process::Command::new("flatc")
        .arg("--binary")
        .arg("-o")
        .arg(out_path.parent().unwrap_or(std::path::Path::new(".")))
        .arg("-I")
        .arg(&schema_dir)
        .arg(schema_dir.join("layout.fbs"))
        .arg(input)
        .status()
        .context("Failed to run flatc — is it installed?")?;

    if status.success() {
        let in_size = std::fs::metadata(input)?.len();
        let out_size = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
        let pct = (out_size * 100)
            .checked_div(in_size)
            .map(|ratio| 100u64.saturating_sub(ratio))
            .unwrap_or(0);
        eprintln!(
            "Converted: {} bytes JSON → {} bytes binary ({}% smaller)",
            in_size, out_size, pct
        );
        Ok(0)
    } else {
        Err(anyhow::anyhow!("flatc conversion failed"))
    }
}

fn cmd_bin2json(input: &PathBuf, output: Option<&std::path::Path>) -> Result<i32> {
    let out_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("voce.json"));

    let schema_dir = find_schema_dir()?;
    let status = std::process::Command::new("flatc")
        .arg("--json")
        .arg("--strict-json")
        .arg("--raw-binary")
        .arg("-o")
        .arg(out_path.parent().unwrap_or(std::path::Path::new(".")))
        .arg("-I")
        .arg(&schema_dir)
        .arg(schema_dir.join("layout.fbs"))
        .arg("--")
        .arg(input)
        .status()
        .context("Failed to run flatc — is it installed?")?;

    if status.success() {
        eprintln!("Wrote {}", out_path.display());
        Ok(0)
    } else {
        Err(anyhow::anyhow!("flatc conversion failed"))
    }
}

#[allow(clippy::too_many_arguments)] // direct mapping from clap subcommand fields
fn cmd_compile(
    file: &PathBuf,
    output: Option<&std::path::Path>,
    debug: bool,
    skip_fonts: bool,
    minify: bool,
    no_cache: bool,
    perf_report: Option<&std::path::Path>,
    report_cache: bool,
) -> Result<i32> {
    // S71 Day 2: when --perf-report is set, time the outer-process work
    // (read, validate, write) and merge into the report alongside the
    // compiler's internal phase timings.
    let invocation_start = std::time::Instant::now();
    let read_start = std::time::Instant::now();
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let read_dur = read_start.elapsed();

    // Validate first
    let validate_start = std::time::Instant::now();
    let val_result = voce_validator::validate(&json).map_err(|e| anyhow::anyhow!("{e}"))?;
    let validate_dur = validate_start.elapsed();
    if val_result.has_errors() {
        eprintln!(
            "voce compile: {} has {} validation error(s). Fix them before compiling.",
            file.display(),
            val_result.error_count()
        );
        voce_validator::formatter::print_terminal(&file.display().to_string(), &val_result);
        return Ok(1);
    }

    // Compile
    // Check cache first (unless --no-cache OR --perf-report — the report
    // is meaningless when the work it would measure was skipped).
    let project_dir = file.parent().unwrap_or(std::path::Path::new("."));
    let mut cache_outcome: &'static str = "miss";
    if !no_cache && perf_report.is_none() {
        let cache = voce_compiler_dom::cache::CompilationCache::new(project_dir);
        if let Some(cached_html) = cache.get(&json) {
            cache_outcome = "hit";
            let out_path = output.map(PathBuf::from).unwrap_or_else(|| {
                let stem = file.file_stem().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("dist/{stem}.html"))
            });
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let size = cached_html.len();
            std::fs::write(&out_path, &cached_html)?;
            eprintln!(
                "✓ Cache hit: {} → {} ({} bytes)",
                file.display(),
                out_path.display(),
                size
            );
            if report_cache {
                emit_cache_report(
                    project_dir,
                    file,
                    &out_path,
                    cache_outcome,
                    size,
                    invocation_start.elapsed(),
                );
            }
            return Ok(0);
        }
    } else if no_cache {
        cache_outcome = "skipped";
    }

    let options = voce_compiler_dom::CompileOptions {
        debug_attrs: debug,
        skip_fonts,
        minify,
        collect_perf_report: perf_report.is_some(),
        ..Default::default()
    };

    let result = voce_compiler_dom::compile(&json, &options)
        .with_context(|| format!("Compilation failed for {}", file.display()))?;

    // Write output
    let out_path = output.map(PathBuf::from).unwrap_or_else(|| {
        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        PathBuf::from(format!("dist/{stem}.html"))
    });

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let write_start = std::time::Instant::now();
    std::fs::write(&out_path, &result.html)?;
    let write_dur = write_start.elapsed();

    // Cache the result
    if !no_cache {
        let cache = voce_compiler_dom::cache::CompilationCache::new(project_dir);
        let _ = cache.put(&json, &result.html);
    }

    // Emit the perf report sidecar if requested.
    if let (Some(report_path), Some(mut report)) = (perf_report, result.perf_report) {
        // Outer-process timings the compiler itself can't see.
        report.add_phase("read_input", read_dur);
        report.add_phase("validate", validate_dur);
        report.add_phase("write_output", write_dur);
        if let Some(parent) = report_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(report_path, report.to_json_pretty())
            .with_context(|| format!("Failed to write perf report to {}", report_path.display()))?;
        eprintln!("  perf report: {}", report_path.display());
    }

    eprintln!(
        "✓ Compiled {} → {} ({} bytes)",
        file.display(),
        out_path.display(),
        result.size_bytes
    );

    if report_cache {
        emit_cache_report(
            project_dir,
            file,
            &out_path,
            cache_outcome,
            result.size_bytes,
            invocation_start.elapsed(),
        );
    }

    Ok(0)
}

/// S71 Day 5: print the cache outcome for this invocation and append a
/// JSONL line to .voce/perf-log.jsonl. The log is opt-in (only written
/// when --report-cache is set) so casual `voce compile` runs don't grow
/// a hidden trail file. Each line is one full record — readable with
/// `jq -s 'group_by(.outcome)' .voce/perf-log.jsonl` or similar.
fn emit_cache_report(
    project_dir: &std::path::Path,
    input: &std::path::Path,
    output_path: &std::path::Path,
    outcome: &str,
    output_bytes: usize,
    elapsed: std::time::Duration,
) {
    let label = match outcome {
        "hit" => "HIT",
        "miss" => "MISS",
        "skipped" => "SKIPPED (--no-cache)",
        other => other,
    };
    eprintln!("  cache: {label}");

    // Append a JSONL line; failure is non-fatal so a read-only project
    // dir doesn't break the compile flow. The perf log is best-effort.
    let log_dir = project_dir.join(".voce");
    if std::fs::create_dir_all(&log_dir).is_err() {
        return;
    }
    let log_path = log_dir.join("perf-log.jsonl");
    let line = serde_json::json!({
        "timestamp_us": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0),
        "input": input.display().to_string(),
        "output": output_path.display().to_string(),
        "outcome": outcome,
        "output_bytes": output_bytes,
        "elapsed_us": elapsed.as_micros() as u64,
    });
    use std::io::Write as _;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "{line}");
    }
}

fn cmd_report(file: &PathBuf, format: &OutputFormat) -> Result<i32> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    let ir: voce_validator::ir::VoceIr =
        serde_json::from_str(&json).with_context(|| "Failed to parse IR")?;

    let validation = voce_validator::validate(&json).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Try to compile and get output size
    let compiled_size =
        voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
            .ok()
            .map(|r| r.size_bytes);

    let report = voce_validator::report::generate_report(
        &file.display().to_string(),
        &ir,
        &validation,
        compiled_size,
    );

    match format {
        OutputFormat::Terminal => voce_validator::report::print_report(&report),
        OutputFormat::Json => voce_validator::report::print_report_json(&report),
    }

    Ok(if validation.has_errors() { 1 } else { 0 })
}

fn cmd_manifest(file: &PathBuf) -> Result<i32> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    let ir: voce_validator::ir::VoceIr =
        serde_json::from_str(&json).with_context(|| "Failed to parse IR")?;

    voce_validator::manifest::print_manifest(&file.display().to_string(), &ir);

    Ok(0)
}

fn cmd_preview(file: &PathBuf) -> Result<i32> {
    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    // Validate
    let val_result = voce_validator::validate(&json).map_err(|e| anyhow::anyhow!("{e}"))?;
    if val_result.has_errors() {
        eprintln!("voce preview: validation errors found");
        voce_validator::formatter::print_terminal(&file.display().to_string(), &val_result);
        return Ok(1);
    }

    // Compile
    let result = voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
        .with_context(|| "Compilation failed")?;

    // Write to temp file
    let preview_dir = std::env::temp_dir().join("voce-preview");
    std::fs::create_dir_all(&preview_dir)?;
    let preview_path = preview_dir.join("index.html");
    std::fs::write(&preview_path, &result.html)?;

    eprintln!(
        "✓ Compiled {} ({} bytes)",
        file.display(),
        result.size_bytes
    );
    eprintln!("  Preview: file://{}", preview_path.display());

    // Try to open in browser
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&preview_path)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&preview_path)
            .spawn();
    }

    Ok(0)
}

fn cmd_generate(prompt: &str, output: Option<&std::path::Path>) -> Result<i32> {
    eprintln!("voce generate: \"{}\"", prompt);
    eprintln!();

    // Find the AI bridge
    let bridge_candidates = [
        PathBuf::from("packages/ai-bridge/dist/cli.js"),
        PathBuf::from("../packages/ai-bridge/dist/cli.js"),
        PathBuf::from("node_modules/.bin/voce-ai-bridge"),
    ];

    let bridge_path = bridge_candidates.iter().find(|p| p.exists()).cloned();

    let bridge = match bridge_path {
        Some(p) => p,
        None => {
            eprintln!(
                "Error: AI bridge not found. Run 'cd packages/ai-bridge && npm install && npm run build' first."
            );
            return Ok(2);
        }
    };

    // Run the bridge
    let status = std::process::Command::new("node")
        .arg(&bridge)
        .arg("generate")
        .arg(prompt)
        .status()
        .context("Failed to run AI bridge")?;

    if !status.success() {
        return Ok(1);
    }

    // The bridge outputs IR JSON to stdout and writes compiled HTML.
    // Copy to output path if specified.
    if let Some(out) = output {
        let generated = std::env::temp_dir().join("voce-generate/generated.voce.json");
        if generated.exists() {
            std::fs::copy(&generated, out)?;
            eprintln!("Saved IR to {}", out.display());
        }
    }

    Ok(0)
}

fn cmd_fix(file: &PathBuf, apply: bool, confidence_str: &str) -> Result<i32> {
    use voce_validator::errors::Confidence;

    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    let threshold = match confidence_str {
        "safe" => Confidence::Safe,
        "suggested" => Confidence::Suggested,
        "risky" => Confidence::Risky,
        other => anyhow::bail!("unknown confidence level: {other}"),
    };

    // Validate first to get diagnostics with their fixes attached.
    let config_dir = file.parent().unwrap_or(std::path::Path::new("."));
    let config = voce_validator::ValidatorConfig::load_from_dir(config_dir);
    let result =
        voce_validator::validate_with_config(&json, &config).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Collect (diagnostic, fix) pairs ordered by node_path so dependent
    // patches apply in document order.
    let mut candidates: Vec<(
        &voce_validator::Diagnostic,
        voce_validator::errors::FixPatch,
    )> = result
        .diagnostics
        .iter()
        .filter_map(|d| voce_validator::build_fix(d).map(|f| (d, f)))
        .collect();
    candidates.sort_by(|a, b| a.0.node_path.cmp(&b.0.node_path));

    if candidates.is_empty() {
        println!("voce fix: no auto-fix proposals available for this file.");
        if result.has_errors() {
            println!(
                "  ({} error(s) need manual attention.)",
                result.error_count()
            );
        }
        return Ok(0);
    }

    let (mut to_apply, mut deferred) = (Vec::new(), Vec::new());
    for (d, fix) in candidates {
        if confidence_meets(&fix.confidence, &threshold) {
            to_apply.push((d, fix));
        } else {
            deferred.push((d, fix));
        }
    }

    println!(
        "voce fix: {} proposal(s) at or below {confidence_str}, {} above threshold",
        to_apply.len(),
        deferred.len()
    );
    println!();

    if !to_apply.is_empty() {
        println!(
            "{}",
            if apply {
                "Applying:"
            } else {
                "Would apply (preview, --apply to write):"
            }
        );
        for (d, fix) in &to_apply {
            println!("  [{}] {}  at {}", fix.confidence, d.code, d.node_path);
            println!("       → {}", fix.preview);
        }
        println!();
    }

    if !deferred.is_empty() {
        println!("Skipped (above {confidence_str} threshold; lower with --confidence):");
        for (d, fix) in &deferred {
            println!("  [{}] {}  at {}", fix.confidence, d.code, d.node_path);
        }
        println!();
    }

    if !apply {
        return Ok(0);
    }

    // Parse, apply each patch in order, write back.
    let mut value: serde_json::Value =
        serde_json::from_str(&json).with_context(|| "failed to parse IR JSON")?;
    let mut applied = 0;
    let mut failed = 0;
    for (d, fix) in &to_apply {
        let mut ok = true;
        for op in &fix.operations {
            if let Err(e) = voce_validator::fixes::apply_op(&mut value, op) {
                eprintln!("  ✗ {} at {}: patch op failed: {e}", d.code, d.node_path);
                ok = false;
                break;
            }
        }
        if ok {
            applied += 1;
        } else {
            failed += 1;
        }
    }

    let updated =
        serde_json::to_string_pretty(&value).with_context(|| "failed to serialize patched IR")?;
    std::fs::write(file, updated).with_context(|| format!("failed to write {}", file.display()))?;

    println!(
        "Wrote {}: {applied} applied, {failed} failed.",
        file.display()
    );

    // Re-validate and show new state.
    let after_json = std::fs::read_to_string(file)?;
    let after = voce_validator::validate_with_config(&after_json, &config)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!(
        "Re-validate: {} ({} → {} errors, {} → {} warnings)",
        if after.has_errors() {
            "✗ still invalid"
        } else {
            "✓ valid"
        },
        result.error_count(),
        after.error_count(),
        result.warning_count(),
        after.warning_count(),
    );

    Ok(if after.has_errors() { 1 } else { 0 })
}

fn confidence_meets(
    actual: &voce_validator::errors::Confidence,
    threshold: &voce_validator::errors::Confidence,
) -> bool {
    use voce_validator::errors::Confidence;
    let rank = |c: &Confidence| match c {
        Confidence::Safe => 0,
        Confidence::Suggested => 1,
        Confidence::Risky => 2,
    };
    rank(actual) <= rank(threshold)
}

fn cmd_deploy(file: &PathBuf, adapter_name: Option<&str>, dry_run: bool) -> Result<i32> {
    use voce_adapter_core::{Adapter, CompiledOutput, ProjectMeta};

    let json = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;

    // Validate
    let val_result = voce_validator::validate(&json).map_err(|e| anyhow::anyhow!("{e}"))?;
    if val_result.has_errors() {
        eprintln!(
            "voce deploy: {} has validation errors. Fix them first.",
            file.display()
        );
        voce_validator::formatter::print_terminal(&file.display().to_string(), &val_result);
        return Ok(1);
    }

    // Compile
    let result = voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
        .with_context(|| "Compilation failed")?;

    // Load config
    let project_dir = file
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let config = voce_adapter_core::load_config(&project_dir).unwrap_or_default();

    // Determine adapter
    let name = adapter_name
        .or(if config.adapter.is_empty() {
            None
        } else {
            Some(config.adapter.as_str())
        })
        .unwrap_or("static");

    let adapter: Box<dyn Adapter> = match name {
        "static" => Box::new(voce_adapter_static::StaticAdapter::default()),
        "vercel" => Box::new(voce_adapter_vercel::VercelAdapter::default()),
        "cloudflare" => Box::new(voce_adapter_cloudflare::CloudflareAdapter::default()),
        "netlify" => Box::new(voce_adapter_netlify::NetlifyAdapter::default()),
        other => {
            eprintln!("Unknown adapter: {other}. Available: static, vercel, cloudflare, netlify");
            return Ok(2);
        }
    };

    // Build compiled output
    let compiled = CompiledOutput {
        html: result.html,
        assets: std::collections::HashMap::new(),
        actions: vec![],
        meta: ProjectMeta {
            name: file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            domain: config.domain.clone(),
            ..Default::default()
        },
    };

    // Prepare bundle
    let bundle = adapter
        .prepare(&compiled, &config)
        .with_context(|| format!("Failed to prepare {} bundle", adapter.name()))?;

    eprintln!("  {}", bundle.summary);

    if dry_run {
        eprintln!("  (dry run — no files written)");
        for path in bundle.files.keys() {
            eprintln!("    {}", path.display());
        }
        return Ok(0);
    }

    // Deploy
    let deploy_result = adapter
        .deploy(&bundle, &config)
        .with_context(|| format!("Deployment via {} failed", adapter.name()))?;

    eprintln!("  {}", deploy_result.message);
    if let Some(url) = &deploy_result.url {
        eprintln!("  Live at: {url}");
    }

    Ok(0)
}

/// Find the schema directory relative to the binary or workspace root.
fn find_schema_dir() -> Result<PathBuf> {
    // Try relative to current directory (workspace root)
    let candidates = [
        PathBuf::from("packages/schema/schemas"),
        PathBuf::from("../packages/schema/schemas"),
        PathBuf::from("../../packages/schema/schemas"),
    ];

    for candidate in &candidates {
        if candidate.join("voce.fbs").exists() {
            return Ok(candidate.clone());
        }
    }

    Err(anyhow::anyhow!(
        "Cannot find schema directory. Run from the project root or set VOCE_SCHEMA_DIR"
    ))
}
