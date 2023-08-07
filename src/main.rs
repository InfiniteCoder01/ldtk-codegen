pub mod definitions;
pub mod schema;
use clap::Parser;
use definitions::*;
use std::path::PathBuf;

pub mod level;
pub mod typedefs;

/// Convert LDTK Project to Rust code
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// LDTK project path
    path: PathBuf,

    /// Output rust file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Preserve case of identifiers
    #[arg(short, long, default_value_t = false)]
    preserve_case: bool,

    /// Derive Serialize and Deserialize
    #[arg(short, long, default_value_t = false)]
    serde: bool,

    /// Make 2D vectors convertable to/from those types (For example raylib::prelude::Vector2 for raylib)
    #[arg(short, long, default_value = "", value_delimiter = ',')]
    vec2s: Vec<String>,

    /// Make colors convertable to/from those types (For example raylib::prelude::Color for raylib)
    #[arg(short, long, default_value = "", value_delimiter = ',')]
    colors: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let preferences = Preferences::new(args.preserve_case, args.serde, args.vec2s, args.colors);
    let mut definitions = RsDefinitions::default();

    let project = serde_json::from_str::<schema::LdtkJson>(
        &std::fs::read_to_string(&args.path).context("Failed to load project file!")?,
    )
    .context("Failed to deserialize project file!")?;

    let mut scope = Scope::new();
    scope.raw("#![allow(dead_code)]");
    if preferences.serde() {
        scope.raw("use serde::{Serialize, Deserialize};");
    }
    typedefs::generate_defs(&preferences, &mut definitions, &project, &mut scope)
        .context("Failed to generate defenitions for LDTK project!")?;
    level::generate_levels(&preferences, &mut definitions, &project, &mut scope)
        .context("Failed to generate levels for LDTK project!")?;

    std::fs::write(
        args.output
            .as_ref()
            .unwrap_or(&args.path.with_extension("rs")),
        scope.to_string(),
    )?;
    Ok(())
}
