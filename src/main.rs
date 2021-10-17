use clap::Clap;
use scanner::{error::Error, operation::Operation, run};
use std::path::Path;

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Create(Create),
    Delete(Delete),
    List,
    Scan(Scan),
}

/// Scan a path, creating a new manifest
#[derive(Clap)]
struct Create {
    /// The path to start the scan
    #[clap(short, long)]
    path: String,
}

/// Delete an existing manifest
#[derive(Clap)]
struct Delete {
    /// Delete the manifest with a particular id
    #[clap(short, long)]
    manifest: i64,
}

/// List existing manifests
#[derive(Clap)]
struct List {}

/// Re-run a scan, create a new manifest and note any differences
#[derive(Clap)]
struct Scan {
    /// Rerun a scan, create a new manifest and compare the results
    #[clap(short, long)]
    manifest: i64,
}

fn main() -> Result<(), Error> {
    let opts = Opts::parse();
    let operation = match opts.subcmd {
        SubCommand::Create(create_matches) => {
            Operation::Index(Path::new(&create_matches.path).to_path_buf())
        }
        SubCommand::Delete(delete_matches) => Operation::DeleteManifest(delete_matches.manifest),
        SubCommand::List => Operation::List,
        SubCommand::Scan(scan_matches) => Operation::Scan(scan_matches.manifest),
    };
    run(operation)
}
