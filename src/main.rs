use std::process;
use clap::Parser;
use walkdir::WalkDir;

//=== Project level imports ===//
mod config;
mod funcs;
use crate::config::{Config,GENERIC_SEARCH_PATHS,MACOS_SEARCH_PATHS,ERR,DbType};
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
    println!("quiet: {} list: {}", args.quiet, args.list);

    let home = std::env::var("HOME").unwrap();

    // Go through default paths for each OS and
    // save a list of all cookie dbs
    let search_roots:String = match std::env::consts::OS {
        "linux" | "freebsd" => {
            String::from(format!(
                "{home}/{}", GENERIC_SEARCH_PATHS[0]
            ))
        },
        "macos" => {
            String::from(format!(
                "{home}/{}", 
                MACOS_SEARCH_PATHS[0]
            ))
        },
        other => {
            eprintln!("Unsupported OS: {}", other);
            process::exit(config.err_exit)
        }
    };

    // https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html
    for entry in WalkDir::new(search_roots).follow_links(false)
       .into_iter()
       .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        if entry.file_type().is_file() {
            let db_type = is_cookie_db(&(entry.path())).unwrap_or_else(|err| {
                debugln!("{ERR} Error reading '{f_name}': {err}");
                return DbType::Unknown;
            });
            if ! matches!(db_type, DbType::Unknown) {
                eprintln!("{f_name}");
            }
        }
    }

    return Ok(());
}
