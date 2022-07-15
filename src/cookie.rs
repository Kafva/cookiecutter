use std::fmt;
use chrono::{TimeZone,Utc};
use crate::types::Cookie;
use crate::{Config,COOKIE_FIELDS,ALL_FIELDS};
use crate::config::ENCRYPTED_VALUE;

/// The output format of cookie fields listed with the `cookies` option
fn field_fmt<T: fmt::Display>(use_name: bool, name: &'static str, value: T) 
 -> String {
    let mut output = String::new();
    if use_name {
        output = if Config::global().nocolor {
            format!("{}: ", name)
        } else {
            format!("\x1b[97;1m{}:\x1b[0m ", name)
        };
    }
    output + &value.to_string()
}

impl Cookie {
    /// Construct a newline separated string with the specified field names
    /// The `fields` parameter is a comma separated string or `All`
    pub fn fields_as_str(&self, fields: &String, use_name: bool) -> String {
        let mut values: Vec<String> = COOKIE_FIELDS.keys().map(|f| {
            // Skip fields not listed in the --fields option
            if !fields.split(",").any(|s| {s==*f || fields == ALL_FIELDS} ) {
               String::from("")
            } else {
                match *f {
                "Host" =>       {
                    field_fmt(use_name, "Host", self.host.to_owned() )
                },
                "Name" =>       {
                    field_fmt(use_name, "Name", self.name.to_owned() )
                },
                "Value" =>      {
                    let has_enc = self.value.is_empty() && 
                             !self.encrypted_value.is_empty();
                    let val = if has_enc {
                        String::from(ENCRYPTED_VALUE)
                     } else {
                        self.value.to_owned()
                     };
                    field_fmt(use_name, "Value", val)
                },
                "Path" =>       {
                    field_fmt(use_name, "Path", self.path.to_owned() )
                },
                "Creation" =>   {
                    field_fmt(use_name, "Creation", 
                        Utc.timestamp(self.creation, 0)
                    )
                },
                "Expiry" =>     {
                    field_fmt(use_name, "Expiry", Utc.timestamp(self.expiry,0))
                },
                "LastAccess" => {
                    field_fmt(use_name, "LastAccess", 
                        Utc.timestamp(self.last_access,0)
                    )
                },
                "HttpOnly" =>   {
                    field_fmt(use_name, "HttpOnly", self.http_only)
                },
                "Secure" =>   {
                    field_fmt(use_name, "Secure", self.secure)
                },
                "SameSite" =>   {
                    let samesite = match self.samesite {
                        2 => "Strict",
                        1 => "Lax",
                        0 => "None",
                        _ => panic!("Unknown SameSite type")
                    };
                    field_fmt(use_name, "SameSite", samesite)
                },
                _ => panic!("Unknown cookie field")
                }
            }}).filter(|f| f != "" ).collect();
        values.sort();
        values.join("\n")
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
