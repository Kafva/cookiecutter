//=== Package imports ===//
use walkdir::WalkDir;
use clap::Parser;

//=== Project imports ===//
mod config;
mod funcs;
mod types;
use crate::config::{Args,Config,CONFIG,SEARCH_DIRS,DB_NAMES};
use crate::funcs::is_cookie_db;
use crate::types::{DbType,CookieDB};

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

    let mut cookie_dbs: Vec<CookieDB> = vec![];

    for search_dir in SEARCH_DIRS {
        // 'home' needs to be cloned since it is referenced in each iteration
        let search_path: String = format!("{}/{}", home.to_owned(), search_dir);

        // We pass a reference of `search_path` since
        // we want to retain ownership of the variable for later use
        for entry in WalkDir::new(&search_path).follow_links(false)
           .into_iter().filter_map(|e| e.ok()) {
            // By filtering on `e.ok()` inaccessible paths are skipped silently

            if entry.file_type().is_file() && 
             DB_NAMES.contains(&entry.file_name().to_string_lossy().as_ref()) {
                let db_type = is_cookie_db(&(entry.path())).unwrap_or_else(|_| {
                    return DbType::Unknown;
                });
                if ! matches!(db_type, DbType::Unknown) {
                    cookie_dbs.push( CookieDB { 
                        path: entry.into_path().to_owned(),
                        typing: db_type 
                    })
                }
            }
        }
    }

    debugln!("{:#?}", cookie_dbs);

    return Ok(());
}
