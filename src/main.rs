use std::collections::HashSet;
//=== Package imports ===//
use walkdir::WalkDir;
use clap::{Parser,CommandFactory};
use chrono::{TimeZone,Utc};

//=== Project imports ===//
mod config;
mod funcs;
mod macros;
mod types;
mod cookie;
mod cookie_db;
use crate::config::{Args,Config,CONFIG,SEARCH_DIRS,DB_NAMES,COOKIE_FIELDS};
use crate::funcs::{cookie_db_type,process_is_running,field_fmt};
use crate::types::{DbType,CookieDB};

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

    let mut cookie_dbs: HashSet<CookieDB> = HashSet::new();

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
                    cookie_dbs.insert( CookieDB {
                        path: entry.into_path().to_owned(),
                        typing: db_type,
                        cookies: vec![]
                    });
                }
            }
        }
    }

    let mut cookie_dbs = Vec::from_iter(cookie_dbs);
    cookie_dbs.sort();

    //== Subcmd: cookies ==//
    if Config::global().list_fields {
        infoln!("Valid fields:");
        for field_name in COOKIE_FIELDS.keys() {
            println!("  {:?}", field_name);
        }
    }
    else if Config::global().list_profiles {
        infoln!("Profiles with a cookie database:");
        cookie_dbs.iter().for_each(|c| {
            println!("  {}", c.path_short(&home));
        });
    }
    else if Config::global().fields != "" && cookie_dbs.len() > 0 {
        for mut cookie_db in cookie_dbs {
            // Skip profiles if a specific --profile was passed
            if Config::global().profile != "" && 
             !cookie_db.path.to_string_lossy()
              .contains(&Config::global().profile) {
                 continue;
            }
            if !Config::global().no_heading {
                infoln!("{}",cookie_db.path_short(&home));
            }

            cookie_db.load_cookies().expect("Failed to load cookies");

            for c in cookie_db.cookies.iter() {
                if Config::global().domain == "" ||
                 c.host.contains(&Config::global().domain) {
                    let mut values: Vec<String> = COOKIE_FIELDS.keys().map(|f| {
                        match *f {
                        "Host" =>       {
                            field_fmt("Host", c.host.to_owned() )
                        },
                        "Name" =>       {
                            field_fmt("Name", c.name.to_owned() )
                        },
                        "Value" =>      {
                            field_fmt("Value", c.value.to_owned())
                        },
                        "Path" =>       {
                            field_fmt("Path", c.path.to_owned() )
                        },
                        "Creation" =>   {
                            field_fmt("Creation", Utc.timestamp(c.creation, 0))
                        },
                        "Expiry" =>     {
                            field_fmt("Expiry", Utc.timestamp(c.expiry,0))
                        },
                        "LastAccess" => {
                            field_fmt("LastAccess", Utc.timestamp(c.last_access,0))
                        },
                        "HttpOnly" =>   {
                            field_fmt("HttpOnly", c.http_only)
                        },
                        _ => panic!("Unknown cookie field")
                    }}).collect();

                    values.sort();
                    println!("{}\n", values.join("\n") );
                }
             }
        }
    } else {
       let mut args_cmd = Args::command();
       args_cmd.print_help().unwrap();
    }

    return Ok(());
}
