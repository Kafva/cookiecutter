use crate::types::{DbType,CookieDB,Cookie};

impl CookieDB {
    /// Fetch the name of the cookies table depending on
    /// the browser type.
    fn table_name(self: &Self) -> &'static str {
        if self.typing == DbType::Firefox {
            "moz_cookies"
        } else {
            "cookies"
        }
    }
    /// Load all cookies from the current `path` into the `cookies` vector
    pub fn load_cookies(self: &mut Self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;

        let mut stmt = conn.prepare(
            &format!("SELECT host,path,name,value FROM {};", self.table_name()),  
        )?;

        let results_iter = stmt.query_map([], |row| {
            // The second parameter to get() denotes
            // the underlying type that the fetched field is expected to have
            Ok(  
                Cookie {
                    host: row.get::<_,String>(0)?,
                    path: row.get::<_,String>(1)?,
                    name: row.get::<_,String>(2)?,
                    value: row.get::<_,String>(3)?
                }
            )
        })?;

        // The query_map() call returns an iterator
        // of results, Ok(), which we need to unwrap
        // before calling collect
        self.cookies = results_iter.filter_map(|r| r.ok() ).collect();

        Ok(())
    }
}
