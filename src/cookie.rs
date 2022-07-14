use std::fmt;
use chrono::{TimeZone,Utc};
use crate::types::Cookie;

impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, concat!(
            "Cookie {{\n  host: \"{}\"\n  name: \"{}\"\n  value: \"{}\"\n  ",
            "path:  \"{}\"\n  creation: \"{}\" ({})\n  expiry: \"{}\" ({})\n}}"),
            self.host, self.name, self.value, self.path,
            Utc.timestamp(self.creation,0), self.creation,
            Utc.timestamp(self.expiry,0), self.expiry
        )
    }
}
