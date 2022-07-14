use std::io::Read; // Enables the use of .read_exact()
use std::io;
use std::fs::File;
use std::path::Path;
use std::fmt;
use sysinfo::{System, SystemExt, RefreshKind};

use crate::types::DbType;
use crate::Config;

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

fn is_db_with_table(conn: &rusqlite::Connection, table_name: &str) -> bool {
    return conn.query_row::<u32,_,_>(
        &format!("SELECT 1 FROM {table_name} LIMIT 1"),
        [],
        |row|row.get(0)
    ).is_ok();
}

/// The output format of cookie fields listed with the `cookies` option
pub fn field_fmt<T: fmt::Display>(name: &'static str, value: T) -> String {
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

