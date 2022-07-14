use std::hash::{Hash, Hasher};
use std::cmp;
/// The PartialEq trait allows us to use `matches!` to check
/// equality between enums
#[derive(Debug,PartialEq)]
pub enum DbType {
    Chrome, Firefox, Unknown
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
    /// The expiry timestamp in UNIX epoch time.
    /// Set to 0 for cookies that expire at the end of a session
    pub expiry: i64,
    /// The last access timestamp in UNIX epoch time
    pub last_access: i64,

    /// If this attribute is set, the client cannot manipulate
    /// the cookie using JS locally in the browser
    pub http_only: bool,

    /// Indicates that the cookie should only be set using
    /// `Set-Cookie` over an encrypted connection (TLS)
    pub secure: bool,

    /// The `SameSite` attribute can be `Lax` (default), `Strict` or `None`
    /// and controls if cookies should be allowed to be sent in requests
    /// to sites where the referrer is from a separate domain.
    ///
    /// Chrome and Firefox use the same enum values:
    ///     Strict == 2
    ///     Lax == 1
    ///     None == 0
    pub samesite: i32
}

#[derive(Debug)]
pub struct CookieDB {
    pub path: std::path::PathBuf,
    pub typing: DbType,
    pub cookies: Vec<Cookie>
}

//== Enable hashing ==//
impl PartialEq for CookieDB {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.typing == other.typing
    }
}
impl Eq for CookieDB {}

impl Hash for CookieDB {
    /// Only considers the filepath of the cookie database
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

//== Enable sorting ==//
impl PartialOrd for CookieDB {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.path.cmp(&other.path))
    }
}
impl Ord for CookieDB {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
