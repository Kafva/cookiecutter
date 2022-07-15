use crypto::pbkdf2::pbkdf2;
use crypto::hmac::Hmac;
use crypto::sha1::Sha1;
use crypto::{ symmetriccipher, buffer, aes, blockmodes };
use crypto::buffer::{ ReadBuffer, WriteBuffer, BufferResult };

use crate::config::COOKIE_FIELDS;
use crate::types::{DbType,CookieDB,Cookie};
use crate::funcs::get_home;

impl CookieDB {
    /// Return the parent of the current path and replaces $HOME with "~".
    /// Returns `path` as is if it is not an absolute path.
    pub fn path_short(&self) -> String {
        if self.path.has_root() {
            self.path.parent().unwrap().to_string_lossy()
                .replace(&get_home(),"~")
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
    fn get_unix_epoch(&self, timestamp:i64) -> i64 {
        if timestamp == 0 {
            0
        } else if self.typing == DbType::Firefox {
            timestamp/1_000_000
        } else {
            (timestamp/1_000_000) - 11_644_473_600
        }
    }

    /// Decrypt a cookie's `encrypted_value` field using a provided key
    fn decrypt_value(&self, enc_value: &[u8], key: &[u8], iv: &[u8]) 
     -> Result<Vec<u8>, symmetriccipher::SymmetricCipherError> {
        let mut decryptor = aes::cbc_decryptor(
            aes::KeySize::KeySize256,
            key,
            iv,
            blockmodes::PkcsPadding
        );
        let mut final_result = Vec::<u8>::new();
        let mut read_buffer = buffer::RefReadBuffer::new(enc_value);
        let mut buffer = [0; 4096];
        let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

        loop {
            let result = decryptor.decrypt(
                &mut read_buffer, &mut write_buffer, true
            )?;
            final_result.extend(
                write_buffer.take_read_buffer().take_remaining()
                .iter().map(|&i| i)
            );
            match result {
                BufferResult::BufferUnderflow => break,
                BufferResult::BufferOverflow => { }
            }
        }
        Ok(final_result)
    }

    /// Attempt to decrypt and load values from the `encrypted_value` column
    /// of a Chrome database.
    /// Adapted from:
    /// https://github.com/bertrandom/chrome-cookies-secure/blob/master/index.js
    fn decrypt_encrypted_values(&self)  {
        match std::env::consts::OS {
        "macos"  => {
            let password = "peanuts";
            let salt = "saltysalt";
            let key_length = 16;
            let mut mac = Hmac::new(Sha1::new(), password.as_bytes());
            let mut derived_key: [u8;16] = [0;16];

            pbkdf2(&mut mac, salt.as_bytes(), key_length, &mut derived_key);

            let iv: [u8; 17] = [0; 17];
            // Note that the first 3 bytes should be skipped
            let cipher = &self.cookies[0].encrypted_value[3..];

            println!("c: {:#?} {}", cipher, cipher.len());

            let plaintext = self.decrypt_value(
                &cipher, 
                &derived_key, 
                &iv
            ).unwrap();

            println!("p: {:#?}", plaintext);
        },
        //"macos"  => {
        //},
        _ => {
        if get_home().starts_with("/mnt/c/Users") { // WSL
        }}
        }
    }

    /// Load all cookies from the current `path` into the `cookies` vector
    pub fn load_cookies(&mut self) -> Result<(), rusqlite::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;
        let field_idx = if self.typing==DbType::Chrome {0} else {1};
        let encrypted_field = if self.typing==DbType::Chrome
                 {"encrypted_value"} else
                 {"NULL"};

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
            Ok(
                Cookie {
                    host: row.get::<_,String>(0).unwrap(),
                    name: row.get::<_,String>(1).unwrap(),
                    value: row.get::<_,String>(2).unwrap(),
                    path: row.get::<_,String>(3).unwrap(),
                    creation: self.get_unix_epoch(
                        row.get::<_,i64>(4).unwrap()
                    ),
                    expiry: self.get_unix_epoch(
                        row.get::<_,i64>(5).unwrap()
                    ),
                    last_access: self.get_unix_epoch(
                        row.get::<_,i64>(6).unwrap()
                    ),
                    http_only: row.get::<_,bool>(7).unwrap(),
                    secure: row.get::<_,bool>(8).unwrap(),
                    samesite: row.get::<_,i32>(9).unwrap(),
                    encrypted_value: row.get::<_,Vec<u8>>(10)
                        .unwrap_or(vec![])
                }
            )
        })?;

        // The query_map() call returns an iterator
        // of results, Ok(), which we need to unwrap
        // before calling collect
        self.cookies = results_iter.filter_map(|r| r.ok() ).collect();

        if self.typing == DbType::Chrome {
            self.decrypt_encrypted_values()
        }

        stmt.finalize().unwrap();
        conn.close().unwrap();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::path::PathBuf;
    use crate::types::{DbType,CookieDB};
    use crate::funcs::get_home;

    #[test]
    fn test_path_short() {
        let mut cdb = CookieDB {
            path: PathBuf::from("./cookies.sqlite"),
            typing: DbType::Chrome,
            cookies: vec![]
        };
        assert_eq!(cdb.path_short(), "./cookies.sqlite");

        cdb.path = PathBuf::from("../../var/Cookies");
        assert_eq!(cdb.path_short(), "../../var/Cookies");

        cdb.path = PathBuf::from(
            format!("{}/.config/chromium/Default/Cookies", get_home())
        );
        assert_eq!(cdb.path_short(), "~/.config/chromium/Default");
    }
}


