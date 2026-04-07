use anyhow::{Context, Result};
use clap::Parser;
use data_ingestion_core::{process, to_format, ContractBuilderConfig, OutputFormat};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "aytch",
    version = "0.1.0",
    about = "Data contract generation CLI — ingest JSON, XML, XSD, CSV, YAML sources into structured data contracts",
    long_about = None,
    after_help = "Examples:\n  aytch --ingest --src schema.json --output ./contracts --type datacontract\n  aytch --ingest --src data.xml --output ./contracts --format yaml\n  aytch --ingest --src employees.xsd --output ./contracts --owner \"hr-team\" --domain \"hr\""
)]
struct Cli {
    /// Ingest a source file and generate data contracts
    #[arg(long = "ingest", short = 'i')]
    ingest: bool,

    /// Path to the source file to analyze
    #[arg(long = "src", short = 's', value_name = "PATH")]
    src: Option<PathBuf>,

    /// Path to the output directory for generated contracts
    #[arg(long = "output", short = 'o', value_name = "PATH")]
    output: Option<PathBuf>,

    /// Type of generation [possible values: datacontract]
    #[arg(long = "type", short = 't', value_name = "TYPE", default_value = "datacontract")]
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

    if cli.ingest {
        run_ingest(&cli)
    } else {
        eprintln!("No action specified. Use --ingest to ingest a source file.");
        eprintln!("Run 'aytch --help' for usage information.");
        std::process::exit(1);
    }
}

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

    // Validate source file exists
    if !src.exists() {
        anyhow::bail!("Source file not found: {}", src.display());
    }
    if !src.is_file() {
        anyhow::bail!("Source path is not a file: {}", src.display());
    }

    // Validate contract type
    if cli.contract_type != "datacontract" {
        anyhow::bail!(
            "Unknown --type '{}'. Currently only 'datacontract' is supported.",
            cli.contract_type
        );
    }

    // Create output directory
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Cannot create output directory: {}", output_dir.display()))?;

    // Read source file
    let content = std::fs::read(src)
        .with_context(|| format!("Failed to read source file: {}", src.display()))?;

    // Get source path string for format detection
    let src_path_str = src.to_string_lossy();
    let src_stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("contract");

    println!("⟳ Ingesting: {}", src.display());

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

    println!("✓ Ingested: {}", src.display());
    println!(
        "✓ Contract: \"{}\" — {} fields",
        contract.name,
        contract.fields.len()
    );
    println!("✓ Writing output to: {}", output_dir.display());

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

        println!("✓ Written: {}", out_path.display());
        written += 1;
    }

    println!(
        "\nDone. {} contract file{} written to {}",
        written,
        if written == 1 { "" } else { "s" },
        output_dir.display()
    );

    Ok(())
}

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
