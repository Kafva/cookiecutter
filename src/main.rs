//=== Package imports ===//
use walkdir::WalkDir;
use strum::IntoEnumIterator;
use clap::{Parser,CommandFactory};

//=== Project imports ===//
mod config;
mod funcs;
mod macros;
mod types;
mod cookie;
mod cookie_db;
use crate::config::{Args,Config,CONFIG,SEARCH_DIRS,DB_NAMES};
use crate::funcs::{cookie_db_type,process_is_running};
use crate::types::{DbType,CookieDB,CookieField};

fn main() -> Result<(),()> {
    // Load command line configuration arguments into a global
    let args: Args = Args::parse();
    let cfg = Config::from_args(args);
    CONFIG.set(cfg).unwrap();

    // Verify that Firefox is not running since it locks the database
    if process_is_running("firefox") {
        errln!("Firefox needs to be closed");
        std::process::exit(Config::global().err_exit);
    }

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
            // The filter is used to skip inaccessible paths 
            if entry.file_type().is_file() && 
             DB_NAMES.contains(&entry.file_name().to_string_lossy().as_ref()) {
                let db_type = cookie_db_type(&(entry.path()))
                    .unwrap_or_else(|_| {
                        return DbType::Unknown;
                    });
                if ! matches!(db_type, DbType::Unknown) {
                    cookie_dbs.push( CookieDB { 
                        path: entry.into_path().to_owned(),
                        typing: db_type, 
                        cookies: vec![]
                    })
                }
            }
        }
    }

    if Config::global().profiles {
        infoln!("Cookie databases:");
        cookie_dbs.iter().for_each(|c| {  
            let db_path = c.path.to_string_lossy().replace(&home,"~");
            println!("  {}", db_path); 
        });
    } else if Config::global().list_fields {
        infoln!("Valid fields:");
        for e in CookieField::iter() {
            println!("  {:?}", e);
        }
    }
    else if Config::global().fields != "" && cookie_dbs.len() > 0 {
        let db = &mut cookie_dbs[0];
        db.load_cookies().expect("Failed to load cookies");


        for (i,c) in db.cookies.iter().enumerate() {
            if c.host == "en.wikipedia.org" {
                println!("{i}: {:#?}", c);
            }
        }
    } else {
       let mut args_cmd = Args::command();
       args_cmd.print_help().unwrap();
    }

    return Ok(());
}
