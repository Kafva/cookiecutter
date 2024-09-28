use std::fmt;

use chrono::{DateTime, TimeZone, Utc};

use crate::config::ENCRYPTED_VALUE;
use crate::{ALL_FIELDS, COOKIE_FIELDS};

#[derive(Debug, Clone)]
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
    pub samesite: i32,

    /// The encrypted value of a cooke, unique to Chrome
    pub encrypted_value: Vec<u8>,
}

impl Cookie {
    /// Construct a newline separated string with the specified field names
    /// The `fields` parameter is a comma separated string or `All`
    pub fn fields_as_str(
        &self,
        fields: &String,
        use_name: bool,
        color: bool,
    ) -> String {
        let mut values: Vec<String> = COOKIE_FIELDS
            .keys()
            .map(|f| {
                // Skip fields not listed in the --fields option
                if !fields.split(",").any(|s| s == *f || fields == ALL_FIELDS) {
                    String::from("")
                } else {
                    self.match_field(*f, use_name, color)
                }
            })
            .filter(|f| f != "")
            .collect();
        values.sort();
        values.join("\n")
    }

    /// Create formatteed output for a given field
    pub fn match_field(
        &self,
        field_name: &str,
        use_name: bool,
        color: bool,
    ) -> String {
        match field_name {
            "Host" => {
                self.field_fmt(color, use_name, "Host", self.host.to_owned())
            }
            "Name" => {
                self.field_fmt(color, use_name, "Name", self.name.to_owned())
            }
            "Value" => {
                let has_enc =
                    self.value.is_empty() && !self.encrypted_value.is_empty();
                let val = if has_enc {
                    String::from(ENCRYPTED_VALUE)
                } else {
                    self.value.to_owned()
                };
                self.field_fmt(color, use_name, "Value", val)
            }
            "Path" => {
                self.field_fmt(color, use_name, "Path", self.path.to_owned())
            }
            "Creation" => self.field_fmt(
                color,
                use_name,
                "Creation",
                Self::date_fmt(self.creation),
            ),
            "Expiry" => self.field_fmt(
                color,
                use_name,
                "Expiry",
                Self::date_fmt(self.expiry),
            ),
            "LastAccess" => self.field_fmt(
                color,
                use_name,
                "LastAccess",
                Self::date_fmt(self.last_access)
            ),
            "HttpOnly" => {
                self.field_fmt(color, use_name, "HttpOnly", self.http_only)
            }
            "Secure" => self.field_fmt(color, use_name, "Secure", self.secure),
            "SameSite" => {
                let samesite = match self.samesite {
                    2 => "Strict",
                    1 => "Lax",
                    -1 | 0 => "None",
                    _ => panic!("Unknown SameSite type"),
                };
                self.field_fmt(color, use_name, "SameSite", samesite)
            }
            _ => panic!("Unknown cookie field"),
        }
    }

    fn date_fmt(epoch: i64) -> DateTime<Utc> {
        match Utc.timestamp_opt(epoch, 0) {
            chrono::offset::LocalResult::Single(s) => s,
            chrono::offset::LocalResult::Ambiguous(e, _) => e,
            _ => DateTime::from_timestamp(0, 0).unwrap()
        }
    }

    /// The output format of cookie fields listed with the `cookies` option
    fn field_fmt<T: fmt::Display>(
        &self,
        color: bool,
        use_name: bool,
        name: &'static str,
        value: T,
    ) -> String {
        let mut output = String::new();
        if use_name {
            output = if !color {
                format!("{}: ", name)
            } else {
                format!("\x1b[97;1m{}:\x1b[0m ", name)
            };
        }
        output + &value.to_string()
    }
}
