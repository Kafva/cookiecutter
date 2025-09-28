use std::cmp;
use std::hash::{Hash, Hasher};

use crate::config::COOKIE_FIELDS;
use crate::cookie::Cookie;
use crate::util::{get_home, DbType};

#[derive(Debug)]
pub struct CookieDB {
    pub path: std::path::PathBuf,
    pub typing: DbType,
    pub cookies: Vec<Cookie>,
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

//== Main impl ==//
impl CookieDB {
    /// Return the parent of the current path and replaces $HOME with "~".
    /// Returns `path` as is if it is not an absolute path.
    pub fn path_short(&self) -> String {
        if self.path.has_root() {
            self.path
                .parent()
                .unwrap()
                .to_string_lossy()
                .replace(&get_home(), "~")
        } else {
            self.path.to_string_lossy().to_string()
        }
    }

    /// Fetch the name of the cookies table depending on
    /// the browser type.
    fn table_name(&self) -> &'static str {
        if self.typing == DbType::Firefox {
            "moz_cookies"
        } else {
            "cookies"
        }
    }

    /// Timestamps are stored internally as UNIX epoch microseconds
    /// for Firefox and as microseconds since Jan 01 1601 in Chrome
    ///
    /// Cookies with a Session-only lifetime will have 0 as their
    /// expiry date in Chrome
    fn get_unix_epoch(&self, timestamp: i64) -> i64 {
        if timestamp == 0 {
            0
        } else if self.typing == DbType::Firefox {
            timestamp / 1_000_000
        } else {
            (timestamp / 1_000_000) - 11_644_473_600
        }
    }

    /// Load all cookies from the current `path` into the `cookies` vector
    pub fn load_cookies(&mut self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open_with_flags(&self.path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let field_idx = if self.typing == DbType::Chrome { 0 } else { 1 };
        let encrypted_field = if self.typing == DbType::Chrome {
            "encrypted_value"
        } else {
            "NULL"
        };

        let query = format!(
            "SELECT {},{},{},{},{},{},{},{},{},{},{} FROM {};",
            COOKIE_FIELDS["Host"][field_idx],
            COOKIE_FIELDS["Name"][field_idx],
            COOKIE_FIELDS["Value"][field_idx],
            COOKIE_FIELDS["Path"][field_idx],
            COOKIE_FIELDS["Creation"][field_idx],
            COOKIE_FIELDS["Expiry"][field_idx],
            COOKIE_FIELDS["LastAccess"][field_idx],
            COOKIE_FIELDS["HttpOnly"][field_idx],
            COOKIE_FIELDS["Secure"][field_idx],
            COOKIE_FIELDS["SameSite"][field_idx],
            encrypted_field,
            self.table_name()
        );
        let mut stmt = conn.prepare(&query)?;
        let results_iter = stmt.query_map([], |row| {
            // The second parameter to get() denotes
            // the underlying type that the fetched field is expected to have
            //
            // We use .unwrap() to get notified explicitly notified
            // of parsing failures
            Ok(Cookie {
                host: row.get::<_, String>(0).unwrap(),
                name: row.get::<_, String>(1).unwrap(),
                value: row.get::<_, String>(2).unwrap(),
                path: row.get::<_, String>(3).unwrap(),
                creation: self.get_unix_epoch(row.get::<_, i64>(4).unwrap()),
                expiry: self.get_unix_epoch(row.get::<_, i64>(5).unwrap()),
                last_access: self.get_unix_epoch(row.get::<_, i64>(6).unwrap()),
                http_only: row.get::<_, bool>(7).unwrap(),
                secure: row.get::<_, bool>(8).unwrap(),
                samesite: row.get::<_, i32>(9).unwrap(),
                encrypted_value: row.get::<_, Vec<u8>>(10).unwrap_or(vec![]),
            })
        })?;

        // The query_map() call returns an iterator
        // of results, Ok(), which we need to unwrap
        // before calling collect
        self.cookies = results_iter.filter_map(|r| r.ok()).collect();

        if self.typing == DbType::Chrome { /* TODO: decrypt() */ }

        stmt.finalize().unwrap();
        conn.close().unwrap();
        Ok(())
    }

    /// Remove all cookies from the underlying database except those
    /// from a domain within the whitelist
    pub fn clean(
        &self,
        whitelist: &Vec<String>,
        apply: bool,
    ) -> Result<(), rusqlite::Error> {
        let field_idx = if self.typing == DbType::Chrome { 0 } else { 1 };

        let query = format!(
            "DELETE FROM {} WHERE {} NOT IN ({});",
            self.table_name(),
            COOKIE_FIELDS["Host"][field_idx],
            whitelist.join(",")
        );

        if apply {
            let conn = rusqlite::Connection::open(&self.path)?;
            conn.execute(&query, rusqlite::params![])?;
            conn.close().unwrap();
        } else {
            println!(" * {query}");
        }
        Ok(())
    }

    /// Delete a cookie with a specific name from a domain or
    /// ALL cookies from a domain if no name is specified.
    /// This call updates both the SQLite store and the
    /// internal `cookies` vector.
    pub fn delete_from_domain(
        &mut self,
        domain: &str,
        name: &str,
    ) -> Result<(), rusqlite::Error> {
        let field_idx = if self.typing == DbType::Chrome { 0 } else { 1 };

        let query = if name.is_empty() {
            format!(
                "DELETE FROM {} WHERE {} == \"{}\";",
                self.table_name(),
                COOKIE_FIELDS["Host"][field_idx],
                domain
            )
        } else {
            format!(
                "DELETE FROM {} WHERE {} == \"{}\" AND {} == \"{}\";",
                self.table_name(),
                COOKIE_FIELDS["Host"][field_idx],
                domain,
                COOKIE_FIELDS["Name"][field_idx],
                name
            )
        };

        // Remove from backing store
        let conn = rusqlite::Connection::open(&self.path)?;
        conn.execute(&query, rusqlite::params![])?;
        conn.close().unwrap();

        if name.is_empty() {
            // Retain all except cookies from the specified domain
            self.cookies.retain(|c| c.host != domain)
        } else {
            // Retain all cookies except those that match `name`
            // and are present on the specified domain
            self.cookies.retain(|c| {
                if c.host == domain {
                    c.name != name
                } else {
                    true
                }
            })
        }

        Ok(())
    }

    /// Deduplicated list of domains stored in the database
    /// Returns a new `String` to dodge BC
    pub fn domains(&self) -> Vec<String> {
        let mut hst_names: Vec<String> =
            self.cookies.iter().map(|c| c.host.to_owned()).collect();
        hst_names.sort();
        hst_names.dedup();
        hst_names
    }

    /// List of cookies for a specific domain
    /// To return a non-reference list of cookies requires that
    /// `Cookie` implements the copy trait
    pub fn cookies_for_domain(&self, domain: &String) -> Vec<Cookie> {
        self.cookies
            .iter()
            .filter(|c| c.host == *domain)
            .map(|c| c.to_owned())
            .collect()
    }

    /// Return a cookie with a specific name from a specific domain
    pub fn cookie_for_domain(
        &self,
        name: &String,
        domain: &String,
    ) -> Option<&Cookie> {
        self.cookies
            .iter()
            .find(|c| c.host == *domain && c.name == *name)
    }
}

#[cfg(test)]
mod tests {
    use crate::cookie_db::CookieDB;
    use crate::path::PathBuf;
    use crate::util::{get_home, DbType};

    #[test]
    fn test_path_short() {
        let mut cdb = CookieDB {
            path: PathBuf::from("./cookies.sqlite"),
            typing: DbType::Chrome,
            cookies: vec![],
        };
        assert_eq!(cdb.path_short(), "./cookies.sqlite");

        cdb.path = PathBuf::from("../../var/Cookies");
        assert_eq!(cdb.path_short(), "../../var/Cookies");

        cdb.path = PathBuf::from(format!(
            "{}/.config/chromium/Default/Cookies",
            get_home()
        ));
        assert_eq!(cdb.path_short(), "~/.config/chromium/Default");
    }
}
