/// The PartialEq trait allows us to use `matches!` to check
/// equality between enums
#[derive(Debug,PartialEq)]
pub enum DbType {
    Chrome, Firefox, Unknown
}

/// The data fields that exist for each cookie
pub enum CookieField {
    Host,
    Name,
    Value,
    Path,
    Creation,
    Expiry
}

#[derive(Debug)]
pub struct Cookie {
    /// The domain that created the cookie 
    pub host: String,
    /// The name of the cookie
    pub name: String,
    /// The value stored in the cookie
    pub value: String,
    /// The URL path of the domain where 
    /// the cookie applies
    pub path: String,
    /// The creation timestamp in UNIX epoch time
    pub creation: i64,
    /// The expiry timestamp in UNIX epoch time
    pub expiry: i64
}

#[derive(Debug)]
pub struct CookieDB {
    pub path: std::path::PathBuf,
    pub typing: DbType,
    pub cookies: Vec<Cookie>
}

//=== Macros ===//
#[macro_export]
macro_rules! errln {
    // Match one or more expressions to this arm
    ( $($x:expr),* ) => (
        eprint!("\x1b[91m!>\x1b[0m ");
        eprintln!($($x)*);
    )
}
#[macro_export]
macro_rules! debugln {
    // Match a fmt literal + one or more expressions
    ( $fmt:literal, $($x:expr),* ) => (
        if Config::global().debug {
            print!("\x1b[94m!>\x1b[0m ");
            println!($fmt, $($x)*);
        }
    );
    // Match one or more expressions without a literal
    ( $($x:expr),* ) => (
        if Config::global().debug {
            print!("\x1b[94m!>\x1b[0m ");
            println!($($x)*);
        }
    )
}

