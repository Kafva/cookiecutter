use std::fmt;

use chrono::{TimeZone,Utc};
use tui::{widgets::ListItem, style::Style};

use crate::{COOKIE_FIELDS,ALL_FIELDS};
use crate::config::ENCRYPTED_VALUE;


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
    pub samesite: i32,

    /// The encrypted value of a cooke, unique to Chrome
    pub encrypted_value: Vec<u8>
}

impl Cookie {
    /// Construct a newline separated string with the specified field names
    /// The `fields` parameter is a comma separated string or `All`
    pub fn fields_as_str(&self, fields: &String, use_name: bool, color: bool) 
     -> String {
        let mut values: Vec<String> = COOKIE_FIELDS.keys().map(|f| {
            // Skip fields not listed in the --fields option
            if !fields.split(",").any(|s| {s==*f || fields == ALL_FIELDS} ) {
               String::from("")
            } else {
                match *f {
                "Host" =>       {
                    self.field_fmt(
                        color, use_name, "Host", self.host.to_owned() 
                    )
                },
                "Name" =>       {
                    self.field_fmt(
                        color, use_name, "Name", self.name.to_owned() 
                    )
                },
                "Value" =>      {
                    let has_enc = self.value.is_empty() && 
                             !self.encrypted_value.is_empty();
                    let val = if has_enc {
                        String::from(ENCRYPTED_VALUE)
                     } else {
                        self.value.to_owned()
                     };
                    self.field_fmt(color, use_name, "Value", val)
                },
                "Path" =>       {
                    self.field_fmt(
                        color, use_name, "Path", self.path.to_owned() 
                    )
                },
                "Creation" =>   {
                    self.field_fmt(color, use_name, "Creation", 
                        Utc.timestamp(self.creation, 0)
                    )
                },
                "Expiry" =>     {
                    self.field_fmt(
                        color, use_name, "Expiry", Utc.timestamp(self.expiry,0)
                    )
                },
                "LastAccess" => {
                    self.field_fmt(color, use_name, "LastAccess", 
                        Utc.timestamp(self.last_access,0)
                    )
                },
                "HttpOnly" =>   {
                    self.field_fmt(color, use_name, "HttpOnly", self.http_only)
                },
                "Secure" =>   {
                    self.field_fmt(color, use_name, "Secure", self.secure)
                },
                "SameSite" =>   {
                    let samesite = match self.samesite {
                        2 => "Strict",
                        1 => "Lax",
                        0 => "None",
                        _ => panic!("Unknown SameSite type")
                    };
                    self.field_fmt(color, use_name, "SameSite", samesite)
                },
                _ => panic!("Unknown cookie field")
                }
            }}).filter(|f| f != "" ).collect();
        values.sort();
        values.join("\n")
    }

    /// The output format of cookie fields listed with the `cookies` option
    fn field_fmt<T: fmt::Display>(&self, color: bool, use_name: bool, 
     name: &'static str, value: T) -> String {
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

    /// Create a `ListItem` vector with one item for each field of the cookie
    pub fn as_list_items(&self) -> Vec<ListItem> {
        self.fields_as_str(&String::from(ALL_FIELDS), true, false).split("\n")
            .map(|f|{
                ListItem::new(f.to_owned()).style(Style::default()) 
        }).collect()
    }
}

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
