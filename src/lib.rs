//! toiletcli is just a collection of common functions that I use in my CLI applications.

/// Enum that contains value to be modified by `parse_flags`.
///
/// # Example
/// ```rust
/// use toiletcli::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (vec!["--colored", "-c"], FlagType::StringFlag(&mut color)),
///     (vec!["--help"], FlagType::SimpleFlag(&mut show_help)),
/// ];
/// ```
#[derive(Debug)]
pub enum FlagType<'a> {
    SimpleFlag(&'a mut bool),
    StringFlag(&'a mut String),
}

/// Array of tuples for `parse_flags` with flag strings and value to be modified.
///
/// # Example
/// ```rust
/// use toiletcli::FlagType;
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (vec!["--colored", "-c"], FlagType::StringFlag(&mut color)),
///     (vec!["--help"], FlagType::SimpleFlag(&mut show_help)),
/// ];
/// ```
pub type Flag<'a> = (Vec<&'a str>, FlagType<'a>);

/// Parses CLI arguments from `sys::env::args`.
///
/// Short flags are two letter flags starting with one dash (`-n`).
/// Long flags are flags starting with two dashes (`--help`).
/// You can combine short `SimpleFlag` flags, eg. `-vAsn` will set `true` to all `-v`, `-A`, `-s`, `-n` flags.
/// **Short flags that use `StringFlag` can't be combined.**
/// # Returns
/// ## Ok
/// All arguments that are not flags.
/// Changes values passed in enums to matching values from args.
///
/// ## Side effects
/// ### `SimpleFlag`
/// ```rust
/// let value: bool = true;  // if flag is specified.
/// let value: bool = false; // if not.
/// ```
///
/// ### `StringFlag`
/// ```rust
/// let value: String = String::from("<value>"); // if flag is specified AND a value is provided,
/// let value: String = String::from("");        // if flag is not specified.
/// ```
///
/// ## Err
/// - On unknown flag;
/// - If `StringFlag` is specified, but no value is provided for it;
/// - If `StringFlag` is combined with some other flag.
///
/// # Example
/// ```rust
/// use std::env::args;
/// use toiletcli::FlagType;
///
/// let mut _args = args();
///
/// let mut color = String::new();
/// let mut show_help = false;
///
/// let mut flags = vec![
///     (vec!["--colored", "-c"], FlagType::StringFlag(&mut color)),
///     (vec!["--help"], FlagType::SimpleFlag(&mut show_help)),
/// ];
///
/// let args = toiletcli::parse_flags(&mut _args.collect(), &mut flags);
/// ```
pub fn parse_flags(args: &Vec<String>, flags: &mut [Flag]) -> Result<Vec<String>, String> {
    let mut args = args.into_iter();
    let mut result: Vec<String> = vec![];
    result.reserve(args.len());

    while let Some(arg) = args.next() {
        let mut chars = arg.chars();

        if chars.next() != Some('-') {
            result.push(arg.clone());
            continue;
        }

        let mut found_long = false;
        let mut first = None;

        while let Some(ch) = chars.next() {
            let mut found = false;

            if found_long {
                break;
            }

            // Longs flags go here.
            if ch == '-' {
                for (flag_strings, flag_value) in &mut *flags {
                    for flag in flag_strings {
                        if arg == *flag {
                            found_long = true;
                            match flag_value {
                                FlagType::SimpleFlag(value) => {
                                    **value = true;
                                }
                                FlagType::StringFlag(value) => {
                                    if let Some(next_arg) = args.next() {
                                        **value = next_arg.clone();
                                    } else {
                                        return Err(format!("No value provided for '{}'", flag));
                                    }
                                }
                            }
                        }
                    }
                }
                if !found_long {
                    return Err(format!("Unknown long flag '{}'", arg));
                }
                break;
            }

            // One letter flags go here. (eg. -aVsd)
            for (flag_strings, flag_value) in &mut *flags {
                for flag in flag_strings {
                    if flag.len() == 2 && ch == flag.chars().last().unwrap() {
                        found = true;
                        match flag_value {
                            FlagType::SimpleFlag(value) => {
                                if let Some(first) = first {
                                    return Err(format!(
                                        "Flag '-{}' requires a value and can't be combined.",
                                        first
                                    ));
                                }
                                **value = true;
                            }
                            FlagType::StringFlag(value) => {
                                if first != None {
                                    return Err(format!(
                                        "Flag '{}' requires a value and can't be combined.",
                                        flag
                                    ));
                                }
                                if let Some(next_arg) = args.next() {
                                    **value = next_arg.clone();
                                    first = Some(ch);
                                } else {
                                    return Err(format!("No value provided for '{}'", flag));
                                }
                            }
                        }
                    }
                }
            }

            if !found {
                return Err(format!("Unknown flag '-{}'", ch));
            }
        }
    }
    return Ok(result);
}

const DIR_CHAR: &'static str = if cfg!(windows) { "\\/" } else { "/" };

/// Gets file name from it's path.
///
/// # Example
/// ```rust
/// let _name = "toilet/bin/program";
/// let name = toiletcli::program_name_from_path(_name);
/// assert_eq!(name, "program");
/// ```
pub fn program_name_from_path(path: &str) -> String {
    for (i, c) in path.chars().rev().enumerate() {
        if DIR_CHAR.contains(c) {
            return String::from(path.split_at(path.len() - i).1);
        }
    }
    return String::from(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let args_vector = vec![
            "program".to_string(),
            "argument_one".to_string(),
            "-aVns".to_string(),
            "--long-specific".to_string(),
            "something".to_string(),
            "argument_two".to_string(),
        ];

        let mut a = false;
        let mut big_v = false;
        let mut n = false;
        let mut s = false;

        let mut long_specific = String::new();

        let mut z = false;
        let mut not_used = String::new();

        let mut flags = vec![
            (vec!["-a"], FlagType::SimpleFlag(&mut a)),
            (vec!["-V"], FlagType::SimpleFlag(&mut big_v)),
            (vec!["-n"], FlagType::SimpleFlag(&mut n)),
            (vec!["-s"], FlagType::SimpleFlag(&mut s)),
            (
                vec!["--long-specific"],
                FlagType::StringFlag(&mut long_specific),
            ),
            (vec!["--not-used"], FlagType::StringFlag(&mut not_used)),
            (vec!["-z"], FlagType::SimpleFlag(&mut z)),
        ];

        println!("{:?}", flags);

        let args = parse_flags(&args_vector, &mut flags).unwrap();

        let args_should_be = vec!["program", "argument_one", "argument_two"];
        let flags_should_be = (
            true,
            true,
            true,
            true,
            &"something".to_string(),
            &"".to_string(),
            false,
        );

        assert_eq!(
            args, args_should_be,
            "args: {:?} should be: {:?}",
            args, args_should_be
        );
        assert_eq!(
            (a, big_v, n, s, &long_specific, &not_used, z),
            flags_should_be,
            "flags: {:?} should be: {:?}",
            (a, big_v, n, s, &long_specific, &not_used, z),
            flags_should_be
        );
    }

    #[test]
    fn program_name() {
        let _name = if cfg!(windows) { "toilet\\bin\\program.exe" } else { "toilet/bin/program" };
        let name = program_name_from_path(_name);
        assert_eq!(name, if cfg!(windows) { "program.exe" } else { "program" });
    }
}
