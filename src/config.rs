//=== Default values ===//
pub static DEBUG: bool = false;

/// The PartialEq trait allows us to use `matches!` to check
/// equality between enums
#[derive(PartialEq)]
pub enum DbType {
    Chrome, Firefox, Unknown
}

// Looking at all files during our search is unnecessarily slow
pub const COOKIE_DB_NAMES: &'static [&str] = &[
    "Cookies",
    "Safe Browsing Cookies",
    "cookies.sqlite"
];

// A `static` lifetime infers that a variable will be defined in 
// the RO section of a binary
pub const MACOS_SEARCH_PATHS: &'static [&str] = &[
    "Library/Application Support/Firefox",
    "Library/Application Support/Chromium",
    "Library/Application Support/BraveSoftware/Brave-Browser"
];

pub const GENERIC_SEARCH_PATHS: &'static [&str] = &[
    ".mozilla/firefox",
    ".config/chromium"
];


//=== Config object ===//
pub struct Config {
    pub err_exit: i32,
}

impl Default for Config {
    fn default() -> Self {
        Config { 
            err_exit: 1,
        }
    }
}

//=== Macros ===//
#[macro_export]
macro_rules! errln {
    // Match one or more expressions to this arm
    ( $($x:expr),* ) => (
        eprint!("\x1b[31m!>\x1b[0m ");
        eprintln!($($x)*);
    )
}
#[macro_export]
macro_rules! debugln {
    // Match one or more expressions to this arm
    ( $($x:expr),* ) => (
        if config::DEBUG {
            print!("\x1b[34m!>\x1b[0m ");
            println!($($x)*);
        }
    )
}


