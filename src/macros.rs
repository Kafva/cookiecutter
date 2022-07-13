#[macro_export]
macro_rules! errln {
    // Match one or more expressions to this arm
    ( $($x:expr),* ) => (
        eprint!("\x1b[91m!>\x1b[0m ");
        eprintln!($($x)*);
    )
}
#[macro_export]
macro_rules! infoln {
    // Match a fmt literal + one or more expressions
    ( $fmt:literal, $($x:expr),* ) => (
        print!("\x1b[94m!>\x1b[0m ");
        println!($fmt, $($x)*);
    );
    // Match one or more expressions without a literal
    ( $($x:expr),* ) => (
        print!("\x1b[94m!>\x1b[0m ");
        println!($($x)*);
    )
}
#[macro_export]
macro_rules! debugln {
    // Match a fmt literal + one or more expressions
    ( $fmt:literal, $($x:expr),* ) => (
        if Config::global().debug {
            print!("\x1b[94m!>\x1b[0m ");
            println!($fmt, $($x)*);
        }
    );
    // Match one or more expressions without a literal
    ( $($x:expr),* ) => (
        if Config::global().debug {
            print!("\x1b[94m!>\x1b[0m ");
            println!($($x)*);
        }
    )
}


