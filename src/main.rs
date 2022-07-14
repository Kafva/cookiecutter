//=== Package imports ===//
use walkdir::WalkDir;
use strum::IntoEnumIterator;
use clap::{Parser,CommandFactory};
use std::str::FromStr;

//=== Project imports ===//
mod config;
mod funcs;
mod macros;
mod types;
mod cookie;
mod cookie_db;
use crate::config::{Args,Config,CONFIG,SEARCH_DIRS,DB_NAMES};
use crate::funcs::{cookie_db_type,process_is_running,field_fmt};
use crate::types::{DbType,CookieDB,CookieField};

fn main() -> Result<(),()> {
    // Load command line configuration arguments into a global
    let args: Args = Args::parse();
    let cfg = Config::from_args(args);
    CONFIG.set(cfg).unwrap();
    if Config::global().debug { eprintln!("{:#?}", Config::global()); }

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

    //== Subcmd: dbs ==//
    if Config::global().dbs {
        infoln!("Cookie databases:");
        cookie_dbs.iter().for_each(|c| {
            let db_path = c.path.to_string_lossy().replace(&home,"~");
            println!("  {}", db_path);
        });
    }
    //== Subcmd: cookies ==//
    else if Config::global().list_fields {
        infoln!("Valid fields:");
        for e in CookieField::iter() {
            println!("  {:?}", e);
        }
    }
    else if Config::global().fields != "" && cookie_dbs.len() > 0 {
        let db = &mut cookie_dbs[0];
        db.load_cookies().expect("Failed to load cookies");

        // 1. Split the fields string and convert each string
        // into a CookieField enum
        let cookie_fields: Vec<CookieField> = Config::global().fields.split(",")
            .map(|f| {
                CookieField::from_str(f).expect("Invalid cookie field name")
            }).collect();

        for c in db.cookies.iter() {
            if Config::global().domain == "" || 
             c.host.contains(&Config::global().domain) {
                // 2. Iterate over the enums for each cookie and
                // fetch the corresponding field value as a string
                let values: Vec<String> = cookie_fields.iter().map(|f| {
                    match f {
                    CookieField::Host =>       { field_fmt("Host", c.host.to_owned() ) },
                    CookieField::Name =>       { field_fmt("Name", c.name.to_owned() ) },
                    CookieField::Value =>      { field_fmt("Value", c.value.to_owned()) },
                    CookieField::Path =>       { field_fmt("Path", c.path.to_owned() ) },

                    CookieField::Creation =>   { field_fmt("Creation", c.creation) },
                    CookieField::Expiry =>     { field_fmt("Expiry", c.expiry) },
                    CookieField::LastAccess => { field_fmt("LastAccess", c.last_access) },

                    CookieField::HttpOnly =>   { field_fmt("HttpOnly", c.http_only) },
                    }
                }).collect();

                println!("{}\n", values.join("\n") );
            }
         }
    } else {
       let mut args_cmd = Args::command();
       args_cmd.print_help().unwrap();
    }

    return Ok(());
}
