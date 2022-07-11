use once_cell::sync::OnceCell;
use clap::Parser;

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

//=== Argument parsing ===//
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Kafva <https://github.com/Kafva>", 
  about = "Cookie manager")]
pub struct Args {
    /// List all cookies across all browsers
    #[clap(short, long)]
    list: bool,

    /// Quiet mode, only print the domains for each cookie when using '-l'
    #[clap(short, long)]
    quiet: bool,

    /// Debug mode
    #[clap(short, long)]
    debug: bool
}

//=== Config object ===//
#[derive(Debug)]
pub struct Config {
    pub err_exit: i32,
    pub debug: bool,
    pub quiet: bool
}

impl Config {
    /// Used to initialise a new config object
    pub fn from_args(args: &Args) -> Self {
        Config { 
            err_exit: 1,
            debug: args.debug,
            quiet: args.quiet
        }
    }
    /// Used to access the global config object in the program
    pub fn global() -> &'static Self {
        CONFIG.get()
            .expect("No globally initialised Config object exists")
    }
}

/// Safe one-shot initialisation of a global
///  https://docs.rs/once_cell/1.4.0/once_cell/
pub static CONFIG: OnceCell<Config> = OnceCell::new();
