//=== Types ===//

/// The PartialEq trait allows us to use `matches!` to check
/// equality between enums
#[derive(Debug,PartialEq)]
pub enum DbType {
    Chrome, Firefox, Unknown
}

#[derive(Debug)]
pub struct CookieDB {
    pub path: std::path::PathBuf,
    pub typing: DbType
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
    // Match a fmt literal + one or more expressions
    ( $fmt:literal, $($x:expr),* ) => (
        if Config::global().debug {
            print!("\x1b[34m!>\x1b[0m ");
            println!($fmt, $($x)*);
        }
    );
    // Match one or more expressions without a literal
    ( $($x:expr),* ) => (
        if Config::global().debug {
            print!("\x1b[34m!>\x1b[0m ");
            println!($($x)*);
        }
    )
}

