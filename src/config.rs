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
    /// List cookies to stdout
    List {
        /// Skip filepath headings
        #[clap(short, long, takes_value = false)]
        no_heading: bool,

        /// Comma separated list of fields to list
        ///     Possible values:
        ///     host,name,value,path,creation,expiry
        #[clap(short, long, default_value = "host")]
        fields: String,
    },
    /// Remove cookies non-interactively
    Clean {
        /// Keep cookies from specified domains
        #[clap(short, long, required = false)]
        whitelist: PathBuf,
    }
}


#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Kafva <https://github.com/Kafva>", 
  about = "Cookie manager")]
pub struct Args {
    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Open a TUI were cookies across all installed browsers can be viewed
    /// and manipulated
    #[clap(short, long)]
    tui: bool,

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
}

impl Default for Config {
    fn default() -> Self {  
        Config {
            err_exit: 1,
            debug: false,
            whitelist: PathBuf::default(),
            no_heading: false,
            fields: String::from("")
        }
    }
}

impl Config {
    /// Initialise a new config object from an Args struct
    pub fn from_args(args: Args) -> Self {
        let mut cfg = Config::default();
        match args.subargs {
            Some(SubArgs::List { no_heading, fields }) => {
                cfg.no_heading = no_heading;
                cfg.fields = fields;
                cfg
            }
            Some(SubArgs::Clean { whitelist }) => {
                cfg.whitelist = whitelist.to_path_buf();
                cfg
            }
            _ => panic!("Unknown argument")
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
