use clap::{Parser, Subcommand};
use once_cell::sync::OnceCell;
use phf::phf_map;

//== Global constants ==//
pub const ENCRYPTED_VALUE: &'static str = "********";
pub const ALL_FIELDS: &'static str = "All";
pub const NO_SELECTION: usize = 9999999;
pub const DEBUG_LOG: &'static str = "cookiecutter.log";
pub const TUI_PRIMARY_COLOR: u8 = 111;
pub const TUI_TEXT_TRUNCATE_LIM: usize = 48;
pub const TUI_SEARCH: &'static str = "Search:";
pub const SQLITE_FILE_ID: &'static str = "SQLite format 3";

pub const DB_NAMES: &'static [&'static str] =
    &["Cookies", "Safe Browsing Cookies", "cookies.sqlite"];

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
    "Library/Application Support/BraveSoftware/Brave-Browser",
];

/// A constant hash map with keys representing each valid Cookie field.
/// Each key maps to a tuple that contains the name of the Chrome and
/// Firefox version of the corresponding field.
pub const COOKIE_FIELDS: phf::Map<&'static str, [&'static str; 2]> = phf_map! {
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

//=== CLI arguments ===//
#[derive(Debug, Subcommand)]
enum SubArgs {
    /// List cookies
    Cookies {
        /// Skip profile headings
        #[clap(short, long, takes_value = false)]
        no_heading: bool,

        /// List valid fields for the --fields option
        #[clap(long, takes_value = false)]
        list_fields: bool,

        /// Comma separated list of fields to list.
        /// If only a single field is supplied, no key names
        /// will be present in the output.
        /// `All` can be supplied as a meta option.
        #[clap(short, long, default_value = "Host,Name")]
        fields: String,

        /// Only include entries matching a specific domain name
        #[clap(short, long, default_value_t)]
        domain: String,
    },
    /// Remove cookies non-interactively
    Clean {
        /// Keep cookies from specified domains
        #[clap(short, long, default_value_t)]
        whitelist: String,

        /// Apply changes
        #[clap(short, long)]
        apply: bool,
    },
    /// Interactive view of cookies across all browsers
    Tui {},
}

#[derive(Parser, Debug)]
#[clap(
    version = "1.0",
    author = "Kafva <https://github.com/Kafva>",
    about = "CLI cookie manager for Firefox and Chromium"
)]
/// https://github.com/clap-rs/clap/blob/v3.2.7/examples/derive_ref/README.md#arg-attributes
/// The `value_parser` trait is required to access an option from the `args`
/// object, this is not usable for subcommands.
pub struct Args {
    /// Output debugging information, writes to `cookiecutter.log` when TUI is active
    #[clap(short, long)]
    debug: bool,

    /// Disable colored output (not applicable for TUI mode)
    #[clap(long)]
    nocolor: bool,

    /// Only include entries from a specific browser profile.
    /// Any unique part of the path to profile can be used as an identifier
    /// e.g. `-p Brave` can be resolved to
    /// `~/.config/BraveSoftware/Brave-Browser/Default`
    #[clap(short, long, default_value_t, value_parser)]
    pub profile: String,

    /// List valid browser profiles for the --profile option
    #[clap(long, takes_value = false, value_parser)]
    pub list_profiles: bool,

    /// Perform all commands on a supplied cookie database
    /// (overrides --profile)
    #[clap(long, short, default_value_t, value_parser)]
    pub file: String,

    #[clap(subcommand)]
    subargs: Option<SubArgs>,
}

//=== Config ===//
#[derive(Debug)]
pub struct Config {
    pub err_exit: i32,

    pub debug: bool,
    pub nocolor: bool,

    // Subcmd: cookies
    pub fields: String,
    pub no_heading: bool,
    pub list_fields: bool,
    pub domain: String,

    // Subcmd: clean
    pub clean: bool,
    pub whitelist: String,
    pub apply: bool,

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
            list_fields: false,
            domain: String::from(""),
            nocolor: false,
            tui: false,
            clean: false,
            apply: false,
        }
    }
}

impl Config {
    /// Initialise a new config object from an Args struct
    pub fn from_args(args: &Args) -> Self {
        let mut cfg = Config::default();
        cfg.nocolor = args.nocolor;
        cfg.debug = args.debug;

        match &args.subargs {
            Some(SubArgs::Cookies {
                no_heading,
                list_fields,
                fields,
                domain,
            }) => {
                cfg.no_heading = *no_heading;
                cfg.list_fields = *list_fields;
                cfg.domain = domain.clone();
                cfg.fields = fields.clone();
                cfg
            }
            Some(SubArgs::Clean { whitelist, apply }) => {
                cfg.clean = true;
                cfg.apply = *apply;
                cfg.whitelist = whitelist.clone();
                cfg
            }
            Some(SubArgs::Tui {}) => {
                cfg.tui = true;
                cfg
            }
            None => cfg,
        }
    }
    /// Used to access the global config object in the program
    pub fn global() -> &'static Self {
        CONFIG
            .get()
            .expect("No globally initialised Config object exists")
    }
}

/// Global configuration object
/// Initialisation from CLI arguments using:
///  https://docs.rs/once_cell/1.4.0/once_cell/
pub static CONFIG: OnceCell<Config> = OnceCell::new();
