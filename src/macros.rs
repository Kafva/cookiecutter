#[macro_export]
macro_rules! msg_prefix {
    ( $x:literal ) => (
        if Config::global().nocolor {
            eprint!("!> ");
        } else {
            eprint!("{}",format!("\x1b[{}m!>\x1b[0m ", $x));
        }
    )
}

#[macro_export]
macro_rules! errln {
    // Match one or more expressions to this arm
    ( $($x:expr),* ) => (
        msg_prefix!("91");
        eprintln!($($x)*);
    )
}
#[macro_export]
macro_rules! infoln {
    // Match a fmt literal + one or more expressions
    ( $fmt:literal, $($x:expr),* ) => (
        msg_prefix!("94");
        eprintln!($fmt, $($x)*);
    );
    // Match one or more expressions without a literal
    ( $($x:expr),* ) => (
        msg_prefix!("94");
        eprintln!($($x)*);
    )
}
#[macro_export]
macro_rules! debugln {
    // Match a fmt literal + one or more expressions
    ( $fmt:literal, $($x:expr),* ) => (
        if Config::global().debug {
            msg_prefix!("94");
            eprintln!($fmt, $($x)*);
        }
    );
    // Match one or more expressions without a literal
    ( $($x:expr),* ) => (
        if Config::global().debug {
            msg_prefix!("94");
            eprintln!($($x)*);
        }
    )
}


