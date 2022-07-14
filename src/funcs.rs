use std::io::Read; // Enables the use of .read_exact()
use std::io;
use std::fs::File;
use std::path::Path;
use std::fmt;

use sysinfo::{System, SystemExt, RefreshKind};
use chrono::{TimeZone,Utc};

use crate::types::{DbType,Cookie};
use crate::{Config,COOKIE_FIELDS};

/// Check if a process is running using the `sysinfo` library
pub fn process_is_running(name: &str) -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::everything()
            .without_cpu()
            .without_disks()
            .without_networks()
            .without_memory()
            .without_components()
            .without_users_list()
    );
    let found = sys.processes_by_exact_name(name)
        .find_map(|_| Some(true)).is_some();
    found
}

/// Construct a newline separated string with the field names
/// and values of a provided cookie (limited to the fields specified
/// in --fields)
pub fn cookie_to_str(cookie: &Cookie) -> String {
    let mut values: Vec<String> = COOKIE_FIELDS.keys().map(|f| {
        // Skip fields not listed in the --fields option
        if !Config::global().fields.split(",").any(|s| {s==*f} ) {
           String::from("")
        } else {
            match *f {
            "Host" =>       {
                field_fmt("Host", cookie.host.to_owned() )
            },
            "Name" =>       {
                field_fmt("Name", cookie.name.to_owned() )
            },
            "Value" =>      {
                field_fmt("Value", cookie.value.to_owned())
            },
            "Path" =>       {
                field_fmt("Path", cookie.path.to_owned() )
            },
            "Creation" =>   {
                field_fmt("Creation", Utc.timestamp(cookie.creation, 0))
            },
            "Expiry" =>     {
                field_fmt("Expiry", Utc.timestamp(cookie.expiry,0))
            },
            "LastAccess" => {
                field_fmt("LastAccess", Utc.timestamp(cookie.last_access,0))
            },
            "HttpOnly" =>   {
                field_fmt("HttpOnly", cookie.http_only)
            },
            "Secure" =>   {
                field_fmt("Secure", cookie.secure)
            },
            "SameSite" =>   {
                let samesite = match cookie.samesite {
                    2 => "Strict",
                    1 => "Lax",
                    0 => "None",
                    _ => panic!("Unknown SameSite type")
                };
                field_fmt("SameSite", samesite)
            },
            _ => panic!("Unknown cookie field")
            }
        }}).filter(|f| f != "" ).collect();
    values.sort();
    values.join("\n")
}

fn is_db_with_table(conn: &rusqlite::Connection, table_name: &str) -> bool {
    return conn.query_row::<u32,_,_>(
        &format!("SELECT 1 FROM {table_name} LIMIT 1"),
        [],
        |row|row.get(0)
    ).is_ok();
}

/// The output format of cookie fields listed with the `cookies` option
fn field_fmt<T: fmt::Display>(name: &'static str, value: T) -> String {
    if Config::global().nocolor {
        format!("{}: {}", name, value)
    } else {
        format!("\x1b[97;1m{}:\x1b[0m {}", name, value)
    }
}

/// Finds all SQLite databases under the given path
/// which feature a non-empty `cookies` or `moz_cookies` table
pub fn cookie_db_type(filepath:&Path) -> Result<DbType,io::Error> {
    let mut f = File::open(filepath)?;
    let mut buf = [0; 15];

    f.read_exact(&mut buf)?;

    for (i,j) in buf.iter().zip("SQLite format 3".as_bytes().iter()) {
        if i != j {
            return Ok(DbType::Unknown);
        }
    }

    let r = rusqlite::Connection::open(filepath);
    if r.is_ok() {
        let conn = r.unwrap();

        if is_db_with_table(&conn, "moz_cookies") {
            conn.close().unwrap();
            return Ok(DbType::Firefox);
        }
        if is_db_with_table(&conn, "cookies") {
            conn.close().unwrap();
            return Ok(DbType::Chrome);
        } else {
            conn.close().unwrap();
        }
    }

    return Ok(DbType::Unknown);
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::cookie_db_type;
    use crate::types::DbType;

    #[test]
    fn test_is_cookie_db() {
        let result = cookie_db_type(Path::new("./moz_cookies.sqlite"));
        assert!(matches!(result.unwrap(), DbType::Firefox));
    }
}

