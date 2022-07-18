use std::io;
use std::{
    env::consts,
    io::{Read,BufRead,Write},
    fs::File,
    path::Path,
    collections::HashSet,
    process::{Command,Stdio}
};

use walkdir::WalkDir;

use sysinfo::{System, SystemExt, RefreshKind};

use crate::cookie_db::CookieDB;
use crate::config::{SEARCH_DIRS,DB_NAMES};

/// The PartialEq trait allows us to use `matches!` to check
/// equality between enums
#[derive(Debug,PartialEq)]
pub enum DbType {
    Chrome, Firefox, Unknown
}

/// Returns /mnt/c/Users/$USER under WSL, otherwise the value of $HOME
pub fn get_home() -> String {
    if std::fs::metadata("/mnt/c/Users").is_ok() {
        format!("/mnt/c/Users/{}", std::env::var("USER").unwrap())
    } else {
        std::env::var("HOME").unwrap()
    }
}

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

/// Search all configured `SEARCH_DIRS` for SQLite databases and
/// add each path to the provided set.
pub fn cookie_dbs_from_profiles(cookie_dbs: &mut HashSet<CookieDB>) {
    let home = get_home();
    for search_dir in SEARCH_DIRS {
        // 'home' needs to be cloned since it is referenced in each iteration
        let search_path: String = format!("{}/{}", home.to_owned(), search_dir);

        // We pass a reference of `search_path` since
        // we want to retain ownership of the variable for later use
        for entry in WalkDir::new(&search_path).follow_links(false)
           .into_iter().filter_map(|e| e.ok()) {
            // The filter is used to skip inaccessible paths
            if entry.file_type().is_file() &&
             DB_NAMES.contains(&entry.file_name().to_string_lossy().as_ref()) {
                let db_type = cookie_db_type(&(entry.path()))
                    .unwrap_or_else(|_| {
                        return DbType::Unknown;
                    });
                if ! matches!(db_type, DbType::Unknown) {
                    cookie_dbs.insert( CookieDB {
                        path: entry.into_path().to_owned(),
                        typing: db_type,
                        cookies: vec![]
                    });
                }
            }
        }
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

/// Parse the domains from a newline separated whitelist into a vector,
/// skipping lines that start with '#'. Each entry will have explicit
/// quotes surrounding it.
pub fn parse_whitelist(filepath: &Path) -> Result<Vec<String>,io::Error> {
    let f = File::open(filepath)?;
    let mut reader = io::BufReader::new(f);

    let mut whitelist = vec![];
    let mut line: String = "".to_string();
    while reader.read_line(&mut line)? > 0 {
       // Skip comments
       let trimmed_line = line.trim();
       if !trimmed_line.starts_with("#") && trimmed_line.len() > 0 {
           // Insert explicit qoutes
           whitelist.push(
               format!("\"{trimmed_line}\"")
           );
       }
       line = "".to_string();
    }
    Ok(whitelist)
}

/// Only applies if `SSH_CONNECTION` is unset.
/// Utilises `xsel` on Linux/BSD.
pub fn copy_to_clipboard(content: String) -> Result<(), io::Error> {
    if std::env::var("SSH_CONNECTION").is_ok() { return Ok(()); }
    match consts::OS {
        "macos" => {
            let mut p = Command::new("/usr/bin/pbcopy")
                .stdin(Stdio::piped()).spawn()?;

            p.stdin.as_mut().unwrap().write_all(content.as_bytes())
        },
        "linux"|"freebsd" => {
            if std::env::var("DISPLAY").is_ok() {  
                let mut p = Command::new("xsel").args(["-i","-b"])
                    .stdin(Stdio::piped()).spawn()?;

                p.stdin.as_mut().unwrap().write_all(content.as_bytes())
            } else {  Ok(()) }
        }
        _ => { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::util::{DbType,cookie_db_type};

    #[test]
    fn test_is_cookie_db() {
        let result = cookie_db_type(Path::new("./moz_cookies.sqlite"));
        assert!(matches!(result.unwrap(), DbType::Firefox));
    }
}

