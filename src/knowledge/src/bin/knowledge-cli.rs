//! CLI binary for the Knowledge SDK — create and manage knowledge packs.

use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use wikilabs_knowledge as knowledge;

#[derive(Parser)]
#[command(name = "knowledge-cli")]
#[command(version = "0.5.0-alpha")]
#[command(about = "Wiki Labs Knowledge SDK CLI — create and manage knowledge packs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new knowledge pack from a template
    CreatePack(CreatePackArgs),
    /// Validate an existing knowledge pack
    Validate(ValidateArgs),
    /// Package a knowledge pack into a .wkl archive
    Package(PackageArgs),
    /// List available templates
    ListTemplates,
}

#[derive(clap::Args)]
struct CreatePackArgs {
    /// Template name (openshift, engineering, documentation)
    #[arg(short, long, default_value = "openshift")]
    template: String,

    /// Output directory for the new pack
    #[arg(short, long, default_value = ".")]
    output: String,

    /// Custom pack name (overrides template name)
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(clap::Args)]
struct ValidateArgs {
    /// Path to the knowledge pack directory
    #[arg(short, long, default_value = ".")]
    path: String,
}

#[derive(clap::Args)]
struct PackageArgs {
    /// Path to the knowledge pack directory
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Output path for the .wkl archive
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::CreatePack(args) => cmd_create_pack(args),
        Commands::Validate(args) => cmd_validate(args),
        Commands::Package(args) => cmd_package(args),
        Commands::ListTemplates => cmd_list_templates(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}

fn cmd_create_pack(args: CreatePackArgs) -> Result<()> {
    let templates = knowledge::sdk::templates::Template::list_names();
    if !templates.contains(&args.template) {
        anyhow::bail!(
            "Unknown template '{}'. Available templates: {}",
            args.template,
            templates.join(", ")
        );
    }

    let output = if let Some(name) = &args.name {
        knowledge::sdk::create_pack::create_pack(&args.template, &args.output, Some(name.clone()))?
    } else {
        knowledge::sdk::create_pack::create_openshift(&args.output)?
    };

    println!("✅ Knowledge pack created at: {output}");
    Ok(())
}

fn cmd_validate(args: ValidateArgs) -> Result<()> {
    let result = knowledge::sdk::validate::validate_pack(&args.path)?;

    println!("📋 Validation Report for: {}", args.path);
    println!("   Name: {}", result.metadata.pack_name);
    println!("   Version: {}", result.metadata.pack_version);
    println!("   Documents: {}", result.document_count);
    println!(
        "   Status: {}",
        if result.is_valid { "VALID" } else { "INVALID" }
    );

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("   ❌ {error}");
        }
        anyhow::bail!("Validation failed with {} error(s)", result.errors.len());
    }

    if !result.warnings.is_empty() {
        for warning in &result.warnings {
            eprintln!("   ⚠️  {warning}");
        }
    }

    println!("✅ Validation passed");
    Ok(())
}

fn cmd_package(args: PackageArgs) -> Result<()> {
    let output_path = args.output.unwrap_or_else(|| "pack.wkl".to_string());

    knowledge::sdk::packager::package_pack(&args.path, &output_path)?;
    println!("✅ Packaged knowledge pack: {output_path}");
    Ok(())
}

fn cmd_list_templates() -> Result<()> {
    let templates = knowledge::sdk::templates::Template::list_names();

    println!("📦 Available templates:\n");
    for name in &templates {
        let description = match name.as_str() {
            "openshift" => "OpenShift / Kubernetes knowledge packs",
            "engineering" => "General engineering knowledge",
            "documentation" => "Documentation / reference knowledge",
            _ => "Unknown template",
        };
        println!("  {} — {description}", name);
    }
    Ok(())
}
