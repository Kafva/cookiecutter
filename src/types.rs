use strum::EnumIter;


/// The PartialEq trait allows us to use `matches!` to check
/// equality between enums
#[derive(Debug,PartialEq)]
pub enum DbType {
    Chrome, Firefox, Unknown
}

/// The data fields that exist for each cookie
#[derive(EnumIter,Debug)]
pub enum CookieField {
    Host,
    Name,
    Value,
    Path,
    Creation,
    Expiry,
    LastAccess,
    HttpOnly
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
    pub expiry: i64,
    /// The last access timestamp in UNIX epoch time
    pub last_access: i64,
    /// If the cookie has HttpOnly set
    pub http_only: bool
}

#[derive(Debug)]
pub struct CookieDB {
    pub path: std::path::PathBuf,
    pub typing: DbType,
    pub cookies: Vec<Cookie>
}

