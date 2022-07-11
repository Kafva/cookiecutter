//=== Package imports ===//
use walkdir::WalkDir;
use clap::Parser;

//=== Project imports ===//
mod config;
mod funcs;
use crate::config::{Args,Config,CONFIG,SEARCH_DIRS,COOKIE_DB_NAMES,DbType};
use crate::funcs::is_cookie_db;

fn main() -> Result<(),()> {
    // Load configuration from argv into a global
    let args: Args = Args::parse();
    let cfg = Config::from_args(&args);
    CONFIG.set(cfg).unwrap();

    // WSL support
    let home: String = if std::fs::metadata("/mnt/c/Users").is_ok() { 
        format!("/mnt/c/Users/{}", std::env::var("USER").unwrap())
    } else {
        std::env::var("HOME").unwrap()
    };

    for search_dir in SEARCH_DIRS {
        // 'home' needs to be cloned since it is referenced anew in each iteration
        let search_path: String = home.to_owned() + search_dir;

        // We pass a reference of `search_path` since
        // we want to retain ownership of the variable for later use
        for entry in WalkDir::new(&search_path).follow_links(false)
           .into_iter().filter_map(|e| e.ok()) {
            // By filtering on `e.ok()` inaccessible paths are skipped silently
            let f_name = entry.file_name().to_str().unwrap();

            if entry.file_type().is_file() && COOKIE_DB_NAMES.contains(&f_name) {
                let f_path = entry.path().to_str().unwrap();
                //debugln!(CONFIG.debug, "Opening: {f_path}");
                let db_type = is_cookie_db(&(entry.path())).unwrap_or_else(|_| {
                    // errln!("Error reading '{f_name}': {err}");
                    return DbType::Unknown;
                });
                if ! matches!(db_type, DbType::Unknown) {
                    eprintln!("{f_path}");
                }
            }
        }

        if Config::global().debug {println!("== {search_path} ==");}

    }

    return Ok(());
}
