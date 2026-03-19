use std::error::Error as StdError;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::io::Error as IoError;
use std::string::FromUtf8Error;

use regex::Error as RegexError;

#[derive(Debug)]
pub enum Error {
    GitInstallation,
    CurrentBranchInvalid,
    InvalidRemote,
    ExitEarly,
    Config(String),
    CommandExecution {
        command: String,
        source: IoError,
    },
    CommandOutputEncoding {
        command: String,
        source: FromUtf8Error,
    },
    InvalidPattern {
        field: &'static str,
        value: String,
        source: RegexError,
    },
    Io(IoError),
}

use self::Error::*;

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Io(ref io_error) => Some(io_error),
            CommandExecution { ref source, .. } => Some(source),
            CommandOutputEncoding { ref source, .. } => Some(source),
            InvalidPattern { ref source, .. } => Some(source),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match *self {
            Io(ref io_error) => io_error.fmt(f),
            ExitEarly => Ok(()),
            Config(ref message) => write!(f, "{}", message),
            CommandExecution {
                ref command,
                ref source,
            } => write!(f, "Failed to execute command `{}`: {}", command, source),
            CommandOutputEncoding { ref command, .. } => {
                write!(f, "Command `{}` produced non-UTF-8 output.", command)
            }
            InvalidPattern {
                field: ref target,
                value: ref pattern,
                ..
            } => write!(f, "Invalid {} pattern: `{}`", target, pattern),
            GitInstallation => {
                write!(f, "Unable to execute 'git' on your machine, please make sure it's installed and on your PATH")
            }
            CurrentBranchInvalid => {
                write!(
                    f,
                    "Please make sure to run git-clean from your base branch (defaults to main)."
                )
            }
            InvalidRemote => {
                write!(f, "That remote doesn't exist, please make sure to use a valid remote (defaults to origin).")
            }
        }
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Io(error)
    }
}
