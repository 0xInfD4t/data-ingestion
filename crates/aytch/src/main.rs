use anyhow::{Context, Result};
use clap::Parser;
use data_ingestion_core::{process, to_format, ContractBuilderConfig, OutputFormat};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "aytch",
    version = "0.1.0",
    about = "Data contract generation CLI — ingest JSON, XML, XSD, CSV, YAML sources into structured data contracts",
    long_about = None,
    after_help = "Examples:\n  aytch --ingest --src schema.json --output ./contracts --type datacontract\n  aytch --ingest --src data.xml --output ./contracts --format yaml\n  aytch --ingest --src employees.xsd --output ./contracts --owner \"hr-team\" --domain \"hr\"\n  aytch --dataquality --src schema.json --output ./expectations --type greatexpectations\n  aytch --dataquality --src ./contracts/ --output ./expectations --recursive\n  aytch --ingest --dataquality --src schema.json --output ./output"
)]
struct Cli {
    /// Ingest a source file and generate data contracts
    #[arg(long = "ingest", short = 'i')]
    ingest: bool,

    /// Generate data quality test suites from a data contract or source file
    #[arg(long = "dataquality", short = 'q', alias = "dq")]
    dataquality: bool,

    /// Process all files in a folder recursively (applies to --ingest and --dataquality)
    #[arg(long = "recursive", short = 'r')]
    recursive: bool,

    /// Skip the 1328 baseline tests, only generate contract-specific suites
    #[arg(long = "no-baseline")]
    no_baseline: bool,

    /// Comma-separated list of suite names to include (default: all)
    #[arg(long = "suites", value_name = "SUITES")]
    suites: Option<String>,

    /// Path to the source file to analyze
    #[arg(long = "src", short = 's', value_name = "PATH")]
    src: Option<PathBuf>,

    /// Path to the output directory for generated contracts
    #[arg(long = "output", short = 'o', value_name = "PATH")]
    output: Option<PathBuf>,

    /// Type of generation [possible values: datacontract, greatexpectations]
    #[arg(long = "type", short = 't', value_name = "TYPE", default_value = "datacontract",
          value_parser = ["datacontract", "greatexpectations"])]
    contract_type: String,

    /// Output format(s) [possible values: json, yaml, xml, csv, all]
    #[arg(long = "format", short = 'f', value_name = "FORMAT", default_value = "all")]
    format: String,

    /// Data contract owner (optional)
    #[arg(long = "owner", value_name = "OWNER")]
    owner: Option<String>,

    /// Data contract domain (optional)
    #[arg(long = "domain", value_name = "DOMAIN")]
    domain: Option<String>,

    /// Disable PII auto-detection
    #[arg(long = "no-pii")]
    no_pii: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match (cli.ingest, cli.dataquality) {
        (true, false) => run_ingest(&cli),
        (false, true) => run_dataquality(&cli),
        (true, true) => {
            // Both flags: ingest first, then generate DQ suites
            run_ingest(&cli)?;
            run_dataquality(&cli)
        }
        (false, false) => {
            eprintln!("No action specified. Use --ingest or --dataquality (or both).");
            eprintln!("Run 'aytch --help' for usage information.");
            std::process::exit(1);
        }
    }
}

// ── Ingest ────────────────────────────────────────────────────────────────────

fn run_ingest(cli: &Cli) -> Result<()> {
    // Validate required args
    let src = cli
        .src
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--src is required for --ingest. Specify the source file path."))?;
    let output_dir = cli
        .output
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--output is required for --ingest. Specify the output directory."))?;

    // Validate contract type
    if cli.contract_type != "datacontract" && cli.contract_type != "greatexpectations" {
        anyhow::bail!(
            "Unknown --type '{}'. Use: datacontract or greatexpectations",
            cli.contract_type
        );
    }

    // Collect source files (supports --recursive for directories)
    let files = collect_source_files(src, cli.recursive)?;

    println!("⟳ Ingesting {} file(s)...", files.len());

    // Create output directory
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Cannot create output directory: {}", output_dir.display()))?;

    for file_path in &files {
        ingest_single_file(file_path, output_dir, cli)?;
    }

    println!(
        "\nDone. Contract file(s) written to {}",
        output_dir.display()
    );
    Ok(())
}

fn ingest_single_file(src: &Path, output_dir: &Path, cli: &Cli) -> Result<()> {
    // Validate source file exists
    if !src.exists() {
        anyhow::bail!("Source file not found: {}", src.display());
    }
    if !src.is_file() {
        anyhow::bail!("Source path is not a file: {}", src.display());
    }

    // Read source file
    let content = std::fs::read(src)
        .with_context(|| format!("Failed to read source file: {}", src.display()))?;

    // Get source path string for format detection
    let src_path_str = src.to_string_lossy();
    let src_stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("contract");

    println!("  ⟳ Ingesting: {}", src.display());

    // Build config
    let config = ContractBuilderConfig {
        version: "1.0.0".to_string(),
        owner: cli.owner.clone(),
        domain: cli.domain.clone(),
        enrich_pii: !cli.no_pii,
        include_nested: true,
        ..Default::default()
    };

    // Process
    let contract = process(&content, None, Some(&src_path_str), config)
        .map_err(|e| anyhow::anyhow!("Failed to ingest {}: {}", src.display(), e))?;

    println!(
        "  ✓ Contract: \"{}\" — {} fields",
        contract.name,
        contract.fields.len()
    );

    // Determine which formats to write
    let formats = resolve_formats(&cli.format)?;

    let mut written = 0;
    for (fmt, ext) in &formats {
        let bytes = to_format(&contract, fmt.clone())
            .map_err(|e| anyhow::anyhow!("Serialization to {} failed: {}", ext, e))?;

        let filename = format!("{}.contract.{}", src_stem, ext);
        let out_path = output_dir.join(&filename);

        std::fs::write(&out_path, &bytes)
            .with_context(|| format!("Failed to write {}", out_path.display()))?;

        println!("  ✓ Written: {}", out_path.display());
        written += 1;
    }

    println!(
        "  ✓ {} contract file{} written",
        written,
        if written == 1 { "" } else { "s" }
    );

    Ok(())
}

// ── Data Quality ──────────────────────────────────────────────────────────────

fn run_dataquality(cli: &Cli) -> Result<()> {
    let src = cli
        .src
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--src is required for --dataquality"))?;
    let output_dir = cli
        .output
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--output is required for --dataquality"))?;

    // Collect source files
    let files = collect_source_files(src, cli.recursive)?;

    println!("⟳ Generating data quality suites for {} file(s)...", files.len());

    for file_path in &files {
        process_dataquality_file(file_path, output_dir, cli)?;
    }

    println!("\nDone. Data quality suites written to {}", output_dir.display());
    Ok(())
}

fn process_dataquality_file(src: &Path, output_dir: &Path, cli: &Cli) -> Result<()> {
    use data_quality_core::{
        generate_all_suites, serialize_suite_set, DqConfig, DqOutputFormat,
    };

    let src_stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("contract");
    let content = std::fs::read(src)
        .with_context(|| format!("Failed to read source file: {}", src.display()))?;
    let src_path_str = src.to_string_lossy();

    // Step 1: Ingest source → DataContract
    println!("  ⟳ Ingesting: {}", src.display());
    let ingest_config = ContractBuilderConfig {
        owner: cli.owner.clone(),
        domain: cli.domain.clone(),
        enrich_pii: !cli.no_pii,
        ..Default::default()
    };
    let contract = process(&content, None, Some(&src_path_str), ingest_config)
        .map_err(|e| anyhow::anyhow!("Failed to ingest {}: {}", src.display(), e))?;

    // Step 2: Build DqConfig
    let enabled_suites = cli.suites.as_ref().map(|s| {
        s.split(',')
            .map(|x| x.trim().to_string())
            .collect::<Vec<_>>()
    });
    let dq_config = DqConfig {
        include_baseline: !cli.no_baseline,
        include_contract_specific: true,
        enabled_suites,
        ..Default::default()
    };

    // Step 3: Generate suites
    let suite_set = generate_all_suites(&contract, &dq_config);
    let total = suite_set.total_test_count;
    println!(
        "  ✓ Generated {} test suites ({} tests) for \"{}\"",
        suite_set.baseline_suites.len() + suite_set.contract_suites.len(),
        total,
        contract.name
    );

    // Step 4: Write output files
    let contract_output_dir = output_dir.join(src_stem);
    std::fs::create_dir_all(&contract_output_dir)?;

    let output_files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
        .map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))?;

    let contract_name_slug = contract.name.to_lowercase().replace(' ', "_");

    for file in &output_files {
        // Determine subdirectory: baseline/ or contract_specific/ or root
        let subdir = if file.filename.starts_with("manifest") || file.filename.starts_with("summary") {
            contract_output_dir.clone()
        } else if file.suite_name.contains(&contract_name_slug) {
            contract_output_dir.join("contract_specific")
        } else {
            contract_output_dir.join("baseline")
        };
        std::fs::create_dir_all(&subdir)?;

        // The filename from serialize_suite_set includes path components like
        // "order/baseline/data_validity_suite.json" — extract just the basename
        let basename = Path::new(&file.filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file.filename);

        let out_path = subdir.join(basename);
        std::fs::write(&out_path, &file.content)?;
        println!("  ✓ Written: {}", out_path.display());
    }

    Ok(())
}

// ── File collection ───────────────────────────────────────────────────────────

fn collect_source_files(src: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    if src.is_file() {
        return Ok(vec![src.to_path_buf()]);
    }
    if src.is_dir() {
        let extensions = ["json", "yaml", "yml", "xml", "xsd", "csv"];
        let mut files = Vec::new();
        collect_files_in_dir(src, &extensions, recursive, &mut files)?;
        if files.is_empty() {
            anyhow::bail!("No supported files found in directory: {}", src.display());
        }
        return Ok(files);
    }
    anyhow::bail!("Source path does not exist: {}", src.display())
}

fn collect_files_in_dir(
    dir: &Path,
    extensions: &[&str],
    recursive: bool,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    files.push(path);
                }
            }
        } else if path.is_dir() && recursive {
            collect_files_in_dir(&path, extensions, recursive, files)?;
        }
    }
    Ok(())
}

// ── Format resolution ─────────────────────────────────────────────────────────

fn resolve_formats(format_str: &str) -> Result<Vec<(OutputFormat, &'static str)>> {
    match format_str.to_lowercase().as_str() {
        "all" => Ok(vec![
            (OutputFormat::Json, "json"),
            (OutputFormat::Yaml, "yaml"),
            (OutputFormat::Xml, "xml"),
            (OutputFormat::Csv, "csv"),
        ]),
        "json" => Ok(vec![(OutputFormat::Json, "json")]),
        "yaml" => Ok(vec![(OutputFormat::Yaml, "yaml")]),
        "xml" => Ok(vec![(OutputFormat::Xml, "xml")]),
        "csv" => Ok(vec![(OutputFormat::Csv, "csv")]),
        other => anyhow::bail!(
            "Unknown --format '{}'. Use: json, yaml, xml, csv, or all",
            other
        ),
    }
}
