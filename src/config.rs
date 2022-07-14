use std::path::PathBuf;
use once_cell::sync::OnceCell;
use clap::{Parser,Subcommand};

// A `static` lifetime infers that a variable will be defined in
// the RO section of a binary
pub const DB_NAMES: &'static [&str] = &[
    "Cookies",
    "Safe Browsing Cookies",
    "cookies.sqlite"
];

pub const SEARCH_DIRS: &'static [&str] = &[
    ".mozilla/firefox",
    ".config/chromium",
    ".config/BraveSoftware/Brave-Browser",

    "Library/Application Support/Firefox/Profiles",
    "Library/Application Support/Chromium",
    "Library/Application Support/BraveSoftware/Brave-Browser",

    "AppData/Roaming/Mozilla/Firefox/Profiles",

    "Library/Application Support/Firefox",
    "Library/Application Support/Chromium",
    "Library/Application Support/BraveSoftware/Brave-Browser"
];

//=== CLI arguments ===//
#[derive(Debug,Subcommand)]
enum SubArgs {
    /// List cookie databases
    Dbs {
    },
    /// List cookies from all databases
    Cookies {
        /// Skip filepath headings
        #[clap(short, long, takes_value = false)]
        no_heading: bool,

        /// List valid fields
        #[clap(short, long, takes_value = false)]
        list_fields: bool,

        /// Comma separated list of fields to list
        #[clap(short, long, default_value = "Host,Name")]
        fields: String,

        /// Only include entries matching a specific domain name
        #[clap(short, long, default_value = "")]
        domain: String,
    },
    /// Remove cookies non-interactively
    Clean {
        /// Keep cookies from specified domains
        #[clap(short, long, required = false)]
        whitelist: PathBuf,
    },
    /// Open a TUI were cookies across all installed browsers can be viewed
    /// and manipulated
    Tui {
    }
}


#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Kafva <https://github.com/Kafva>",
  about = "Cookie manager")]
pub struct Args {
    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Disable colored output (not applicable for TUI mode)
    #[clap(long)]
    nocolor: bool,

    #[clap(subcommand)]
    subargs: Option<SubArgs>
}


//=== Config ===//
#[derive(Debug)]
pub struct Config {
    pub err_exit: i32,
    pub debug: bool,
    pub whitelist: PathBuf,
    pub no_heading: bool,
    pub fields: String,
    pub dbs: bool,
    pub list_fields: bool,
    pub domain: String,
    pub nocolor: bool,
    pub tui: bool
}

impl Default for Config {
    fn default() -> Self {
        Config {
            err_exit: 1,
            debug: false,
            whitelist: PathBuf::default(),
            no_heading: false,
            fields: String::from(""),
            dbs: false,
            list_fields: false,
            domain: String::from(""),
            nocolor: false,
            tui: false
        }
    }
}

impl Config {
    /// Initialise a new config object from an Args struct
    pub fn from_args(args: Args) -> Self {
        let mut cfg = Config::default();
        cfg.nocolor = args.nocolor;
        cfg.debug   = args.debug;

        match args.subargs {
            Some(SubArgs::Dbs {  }) => {
                cfg.dbs = true; cfg
            }
            Some(SubArgs::Cookies {
                no_heading, list_fields, fields, domain
            }) => {
                cfg.no_heading = no_heading;
                cfg.list_fields = list_fields;
                cfg.domain = domain;
                cfg.fields = fields; cfg
            }
            Some(SubArgs::Clean { whitelist }) => {
                cfg.whitelist = whitelist.to_path_buf(); cfg
            }
            Some(SubArgs::Tui {  }) => { cfg }
            None => { cfg }
        }
    }
    /// Used to access the global config object in the program
    pub fn global() -> &'static Self {
        CONFIG.get()
            .expect("No globally initialised Config object exists")
    }
}

/// Global configuration object
/// Initialisation from CLI arguments using:
///  https://docs.rs/once_cell/1.4.0/once_cell/
pub static CONFIG: OnceCell<Config> = OnceCell::new();
