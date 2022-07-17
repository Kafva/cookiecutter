use std::collections::HashSet;
use std::path;

use clap::{Parser,CommandFactory};

//=== Project imports ===//
mod config;
mod util;
mod macros;
mod cookie;
mod cookie_db;
mod state;
mod tui;
use crate::config::{
    Args,Config,CONFIG,COOKIE_FIELDS,ALL_FIELDS,DEBUG_LOG
};
use crate::util::{cookie_db_type,process_is_running,cookie_dbs_from_profiles,parse_whitelist};
use crate::cookie_db::CookieDB;
use crate::tui::run;

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

    let mut cookie_dbs: HashSet<CookieDB> = HashSet::new();

    // Parse a custom db if a --file was provided
    if Config::global().file != "" {
        let custom_db_path = path::PathBuf::try_from(&Config::global().file)
                .expect("Could not create PathBuf from provided --file");
        let typing = cookie_db_type(&custom_db_path.as_path())
                .expect("Failed to determine database type of --file argument");
        cookie_dbs.insert( CookieDB {
            path:  custom_db_path,
            typing,
            cookies: vec![]
        });
    } else {
        // Fetch a set of all cookie dbs on the system
        cookie_dbs_from_profiles(&mut cookie_dbs);
    }
    let mut cookie_dbs = Vec::from_iter(cookie_dbs);
    cookie_dbs.sort();


    // Explicitly note if an invalid --profile was specified
    if Config::global().profile != "" &&
     !cookie_dbs.iter().any(|c|
      c.path.to_string_lossy().to_owned()
        .contains(&Config::global().profile)
    ) {
        errln!("No profile matching '{}' found", Config::global().profile);
        std::process::exit(Config::global().err_exit);
    }


    if Config::global().list_profiles {
        infoln!("Profiles with a cookie database:");
        cookie_dbs.iter().for_each(|c| {
            println!("  {}", c.path_short());
        });
    }
    //== Subcmd: cookies ==//
    else if Config::global().list_fields {
        infoln!("Valid fields:");
        for field_name in COOKIE_FIELDS.keys() {
            println!("  {:?}", field_name);
        }
    }
    else if Config::global().fields != "" && cookie_dbs.len() > 0 {
        let multiple_fields = Config::global().fields
            .find(",").is_some() || Config::global().fields == ALL_FIELDS;

        for mut cookie_db in cookie_dbs {
            // Skip profiles if a specific --profile was passed
            if Config::global().profile != "" &&
             !cookie_db.path.to_string_lossy()
              .contains(&Config::global().profile) {
                 continue;
            }
            // Skip profile headings if --no-heading
            if !Config::global().no_heading {
                infoln!("{}",cookie_db.path_short());
            }
            // Load all fields from each cookie database
            cookie_db.load_cookies().expect("Failed to load cookies");
            let mut output_str = String::new();

            for c in cookie_db.cookies.iter() {
                // Skip domains if a specific --domain was passed
                if Config::global().domain == "" ||
                 c.host.contains(&Config::global().domain) {

                    output_str = output_str +
                        &c.fields_as_str(&Config::global().fields,
                            multiple_fields
                        ) + "\n";

                    if multiple_fields {
                        // Skip blankline if only one field is being printed
                        output_str = output_str + "\n"
                    }
                }
            }
            print!("{output_str}");
        }
    }
    //== Subcmd: clean ==//
    else if Config::global().clean {
        let mut whitelist = vec![];
        if Config::global().whitelist != "" {
           let filepath = path::PathBuf::try_from(&Config::global().whitelist)
               .expect("Failed to convert whitelist into a PathBuf");
           whitelist = parse_whitelist(&filepath.as_path())
               .expect("Failed to parse whitelist");
        }

        for cookie_db in cookie_dbs {
            // Skip profiles if a specific --profile was passed
            if Config::global().profile != "" &&
             !cookie_db.path.to_string_lossy()
              .contains(&Config::global().profile) {
                 continue;
            }
            infoln!("Cleaning {}", cookie_db.path_short());
            cookie_db.clean(&whitelist, Config::global().apply)
                .expect("Failed to delete cookies from database");
        }
        if Config::global().apply {
            infoln!("== Deletions committed ==");
        } else {
            infoln!("To perform deletions, pass `--apply`");
        }
    } 
    //== Subcmd: tui ==//
    else if Config::global().tui {
        // Delete any previous debug log
        std::fs::remove_file(DEBUG_LOG).unwrap_or_default();

        cookie_dbs.iter_mut().for_each(|c| 
           c.load_cookies().expect("Failed to load cookies")
        );
        run(&cookie_dbs).expect("Failed to create TUI");
    } else {
       let mut args_cmd = Args::command();
       args_cmd.print_help().unwrap();
    }

    return Ok(());
}
