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
    #[arg(long, default_value_t = false)]
    preserve_case: bool,

    /// Derive serde Serialize and Deserialize
    #[arg(long, default_value_t = false)]
    serde: bool,

    /// Use this as a vector type (`raylib::prelude::Vector2` for example)
    /// ldtk_module::VectorImpl has to be implemented for this type
    /// You can append <T> to the end of the type for generic vectors (`speedy::dimen::Vector2<T>`)
    #[arg(long)]
    vector: Option<String>,

    /// Use this as a color type (`raylib::prelude::Color` for example)
    /// ldtk_module::ColorImpl has to be implemented for this type
    #[arg(long)]
    color: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let preferences = Preferences {
        preserve_case: args.preserve_case,
        serde: args.serde,
        vector: args.vector,
        color: args.color,
    };
    let mut definitions = RsDefinitions::default();

    let project = serde_json::from_str::<schema::LdtkJson>(
        &std::fs::read_to_string(&args.path).context("Failed to load project file!")?,
    )
    .context("Failed to deserialize project file!")?;

    let mut scope = Scope::new();
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
