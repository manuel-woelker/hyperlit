use super::VERSION_STRING;
use clap::{Parser, Subcommand};

/// Documentation builder for documentation embedded in source files
#[derive(Parser, Debug, PartialEq)]
#[command(version = VERSION_STRING, about, long_about = None)]
pub struct HyperlitCliArgs {
    #[command(subcommand)]
    pub command: Option<HyperlitCliCommands>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum HyperlitCliCommands {
    /// Create and initialize a new hyperlit project
    Init {},
    /// Watch for filesystem changes and rebuild automatically
    Watch {},
}

#[cfg(test)]
mod tests {
    use crate::arguments::{HyperlitCliArgs, HyperlitCliCommands};
    use clap::Parser;

    #[test]
    fn test_parse_help() {
        let result = HyperlitCliArgs::try_parse_from(["hyperlit", "--help"]).unwrap_err();
        assert_eq!(result.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_parse_version() {
        let result = HyperlitCliArgs::try_parse_from(["hyperlit", "--version"]).unwrap_err();
        assert_eq!(result.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_parse_init() {
        let result = HyperlitCliArgs::try_parse_from(["hyperlit", "init"]).unwrap();
        assert_eq!(
            result,
            HyperlitCliArgs {
                command: Some(HyperlitCliCommands::Init {})
            }
        );
    }

    #[test]
    fn test_parse_watch() {
        let result = HyperlitCliArgs::try_parse_from(["hyperlit", "watch"]).unwrap();
        assert_eq!(
            result,
            HyperlitCliArgs {
                command: Some(HyperlitCliCommands::Watch {})
            }
        );
    }

    #[test]
    fn test_parse_nothing() {
        let result = HyperlitCliArgs::try_parse_from(["hyperlit"]).unwrap();
        assert_eq!(result, HyperlitCliArgs { command: None });
    }

    #[test]
    fn test_parse_unknown() {
        let result = HyperlitCliArgs::try_parse_from(["hyperlit", "invalid"]).unwrap_err();
        assert_eq!(result.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }
}
