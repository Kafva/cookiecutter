use once_cell::sync::OnceCell;
use clap::{Parser,Subcommand};
use phf::phf_map;

//== Global constants ==//
pub const DB_NAMES: &'static [&'static str] = &[
    "Cookies",
    "Safe Browsing Cookies",
    "cookies.sqlite"
];

pub const SEARCH_DIRS: &'static [&'static str] = &[
    ".mozilla/firefox",
    ".config/chromium",
    ".config/BraveSoftware/Brave-Browser",

    "Library/Application Support/Firefox/Profiles",
    "Library/Application Support/Chromium",
    "Library/Application Support/BraveSoftware/Brave-Browser",

    "AppData/Roaming/Mozilla/Firefox/Profiles",

    "Library/Application Support/Firefox",
    "Library/Application Support/Chromium",
    "Library/Application Support/BraveSoftware/Brave-Browser"
];

/// A constant hash map with keys representing each valid Cookie field.
/// Each key maps to a tuple that contains the name of the Chrome and
/// Firefox version of the corresponding field.
pub const COOKIE_FIELDS: phf::Map<&'static str, [&'static str; 2]> = phf_map!{
    "Host"       => ["host_key",         "host"],
    "Name"       => ["name",             "name"],
    "Value"      => ["value",            "value"],
    "Path"       => ["path",             "path"],
    "Creation"   => ["creation_utc",     "creationTime"],
    "Expiry"     => ["expires_utc",      "expiry"],
    "LastAccess" => ["last_access_utc",  "lastAccessed"],
    "HttpOnly"   => ["is_httponly",      "isHttpOnly"],
    "Secure"     => ["is_secure",        "isSecure"],
    "SameSite"   => ["samesite",         "sameSite"],
};

pub const ENCRYPTED_VALUE: &'static str = "********";

//=== CLI arguments ===//
#[derive(Debug,Subcommand)]
enum SubArgs {
    /// List cookies
    Cookies {
        /// Skip profile headings
        #[clap(short, long, takes_value = false)]
        no_heading: bool,

        /// List valid fields for the --fields option
        #[clap(long, takes_value = false)]
        list_fields: bool,

        /// List valid browser profiles for the --profile option
        #[clap(long, takes_value = false)]
        list_profiles: bool,

        /// Comma separated list of fields to list
        #[clap(short, long, default_value = "Host,Name")]
        fields: String,

        /// Only include entries matching a specific domain name
        #[clap(short, long, default_value_t)]
        domain: String,

        /// Only include entries from a specific browser profile.
        /// The profile can be given as a partial path to differentiate
        /// between profiles with the same name in different browsers.
        #[clap(short, long, default_value_t)]
        profile: String,
    },
    /// Remove cookies non-interactively
    Clean {
        /// Keep cookies from specified domains
        #[clap(short, long, default_value_t)]
        whitelist: String
    },
    /// Open a TUI were cookies across all installed browsers can be viewed
    /// and manipulated
    Tui {
    }
}


#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Kafva <https://github.com/Kafva>",
  about = "Cookie manager")]
/// https://github.com/clap-rs/clap/blob/v3.2.7/examples/derive_ref/README.md#arg-attributes
pub struct Args {
    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Disable colored output (not applicable for TUI mode)
    #[clap(long)]
    nocolor: bool,

    /// Perform all commands on a supplied cookie database
    /// (overrides --profile)
    #[clap(long, short, default_value_t)]
    file: String,

    #[clap(subcommand)]
    subargs: Option<SubArgs>
}


//=== Config ===//
#[derive(Debug)]
pub struct Config {
    pub err_exit: i32,

    pub debug: bool,
    pub file: String,
    pub nocolor: bool,

    // Subcmd: cookies
    pub fields: String,
    pub no_heading: bool,
    pub list_profiles: bool,
    pub list_fields: bool,
    pub domain: String,
    pub profile: String,

    // Subcmd: clean
    pub clean: bool,
    pub whitelist: String,

    // Subcmd: tui
    pub tui: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            err_exit: 1,
            debug: false,
            whitelist: String::from(""),
            no_heading: false,
            fields: String::from(""),
            list_profiles: false,
            list_fields: false,
            domain: String::from(""),
            nocolor: false,
            tui: false,
            profile: String::from(""),
            file: String::from(""),
            clean: false
        }
    }
}

impl Config {
    /// Initialise a new config object from an Args struct
    pub fn from_args(args: Args) -> Self {
        let mut cfg = Config::default();
        cfg.nocolor = args.nocolor;
        cfg.debug   = args.debug;
        cfg.file    = args.file;

        match args.subargs {
            Some(SubArgs::Cookies {
                no_heading, list_fields, fields, domain, list_profiles, profile
            }) => {
                cfg.no_heading = no_heading;
                cfg.list_fields = list_fields;
                cfg.domain = domain;
                cfg.list_profiles = list_profiles;
                cfg.profile = profile;
                cfg.fields = fields; cfg
            }
            Some(SubArgs::Clean { whitelist }) => {
                cfg.clean = true;
                cfg.whitelist = whitelist; cfg
            }
            Some(SubArgs::Tui {  }) => { cfg }
            None => { cfg }
        }
    }
    /// Used to access the global config object in the program
    pub fn global() -> &'static Self {
        CONFIG.get()
            .expect("No globally initialised Config object exists")
    }
}

/// Global configuration object
/// Initialisation from CLI arguments using:
///  https://docs.rs/once_cell/1.4.0/once_cell/
pub static CONFIG: OnceCell<Config> = OnceCell::new();
