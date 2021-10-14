use clap::{App, Arg};
use scanner::{operation::Operation, run};
use std::path::Path;

fn main() {
    let matches = App::new("Scanner")
        .version("0.1")
        .author("Richard Bradshaw")
        .about("Tripwire like thing")
        .subcommand(
            App::new("index").arg(
                Arg::new("path")
                    .long("path")
                    .required(true)
                    .takes_value(true)
                    .short('p'),
            ),
        )
        .subcommand(App::new("list"))
        .subcommand(
            App::new("manifest").arg(
                Arg::new("delete")
                    .long("delete")
                    .takes_value(true)
                    .short('d'),
            ),
        )
        .subcommand(
            App::new("scan")
                .arg(
                    Arg::new("manifest")
                        .long("manifest")
                        .required(true)
                        .takes_value(true)
                        .short('m'),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .required(true)
                        .takes_value(true)
                        .short('p'),
                ),
        )
        .get_matches();

    let operation = match matches.subcommand() {
        Some(("index", index_matches)) => {
            let path = Path::new(index_matches.value_of("path").unwrap());
            Operation::Index(path.to_path_buf())
        }
        Some(("list", _)) => Operation::List,
        Some(("manifest", manifest_matches)) => {
            let manifest_id = manifest_matches
                .value_of("delete")
                .unwrap()
                .parse()
                .unwrap();
            Operation::DeleteManifest(manifest_id)
        }
        Some(("scan", scan_matches)) => {
            let manifest: i64 = scan_matches.value_of("manifest").unwrap().parse().unwrap();
            let path = Path::new(scan_matches.value_of("path").unwrap());
            Operation::Scan(manifest, path.to_path_buf())
        }
        _ => unreachable!(),
    };

    run(operation).unwrap();
}
