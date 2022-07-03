use std::process;
use clap::Parser;
use walkdir::WalkDir;

//=== Project level imports ===//
mod config;
mod funcs;
use crate::config::{Config,GENERIC_SEARCH_PATHS,MACOS_SEARCH_PATHS,COOKIE_DB_NAMES,DbType,DEBUG};
use crate::funcs::is_cookie_db;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Kafva <https://github.com/Kafva>", 
  about = "Cookie manager")]
struct Args {
    /// List all cookies across all browsers
    #[clap(short, long)]
    list: bool,

    /// Quiet mode, only print the domains for each cookie when using '-l'
    #[clap(short, long)]
    quiet: bool
}

fn main() -> Result<(),()> {
    let config = Config::default();
    let args: Args = Args::parse();
    if DEBUG {println!("quiet: {} list: {}\n", args.quiet, args.list);}
    // https://docs.rs/once_cell/1.4.0/once_cell/
    // DEBUG = args.quiet;

    let home = std::env::var("HOME").unwrap();

    // Go through default paths for each OS and
    // save a list of all cookie dbs
    let search_roots:&'static[&'static str] = match std::env::consts::OS {
        "linux" | "freebsd" => {
            GENERIC_SEARCH_PATHS
        },
        "macos" => {
            MACOS_SEARCH_PATHS
        },
        other => {
            eprintln!("Unsupported OS: {}", other);
            process::exit(config.err_exit)
        }
    };

    // https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html
    for search_root in search_roots {
        if DEBUG {println!("== {search_root} ==");}
        for entry in WalkDir::new(format!("{home}/{search_root}")).follow_links(false)
           .into_iter().filter_map(|e| e.ok()) {
            // By filtering on `e.ok()` inaccessible paths are skipped silently
            let f_name = entry.file_name().to_str().unwrap();

            if entry.file_type().is_file() && COOKIE_DB_NAMES.contains(&f_name) {
                let f_path = entry.path().to_str().unwrap();
                debugln!("Opening: {f_path}");
                let db_type = is_cookie_db(&(entry.path())).unwrap_or_else(|_| {
                    // errln!("Error reading '{f_name}': {err}");
                    return DbType::Unknown;
                });
                if ! matches!(db_type, DbType::Unknown) {
                    eprintln!("{f_path}");
                }
            }
        }
    }

    return Ok(());
}
