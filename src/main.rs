use clap::{ArgGroup, Args, CommandFactory, Parser, Subcommand, ValueEnum, error::ErrorKind};
use monke::SaveFile;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(version, about, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Decrypt and decompress a save file.
    #[command(visible_alias = "u")]
    Unpack(TransformArgs),

    /// Compress and encrypt save data.
    #[command(visible_alias = "p")]
    Pack {
        #[command(flatten)]
        args: TransformArgs,

        /// Preserve metadata from an existing encoded save.
        #[arg(short, long)]
        template: Option<PathBuf>,
    },

    /// Print save metadata without decrypting it.
    Inspect {
        /// Encoded save file to inspect.
        input: PathBuf,
    },
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Self::Unpack(_) => "unpack",
            Self::Pack { .. } => "pack",
            Self::Inspect { .. } => "inspect",
        }
    }
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("password source")
        .multiple(false)
        .args(["game", "password"])
))]
struct TransformArgs {
    /// Input file to transform.
    input: PathBuf,

    /// Destination file.
    #[arg(short, long)]
    output: PathBuf,

    /// Use the known password for a supported game.
    #[arg(long, value_enum)]
    game: Option<Game>,

    /// Encryption password.
    #[arg(long)]
    password: Option<String>,

    /// Replace the destination if it exists.
    #[arg(short, long)]
    force: bool,
}

impl TransformArgs {
    fn password(&self, operation: Operation) -> Result<&str, CliError> {
        match (&self.game, &self.password) {
            (Some(game), None) => Ok(game.password(operation)),
            (None, Some(password)) => Ok(password),
            _ => Err(CliError::PasswordSource),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum Game {
    Battd,
    Btd6,
    TowerKeepers,
}

impl Game {
    fn password(&self, operation: Operation) -> &'static str {
        // Thanks to https://github.com/Roshi617 for finding the BATTD passwords.
        match (self, operation) {
            (Self::Battd, Operation::Unpack) => "$LXvKp90n$Cc0GTX2nY5",
            (Self::Battd, Operation::Pack) => "16",
            (Self::Btd6, _) => "11",
            (Self::TowerKeepers, _) => "8",
        }
    }
}

#[derive(Clone, Copy)]
enum Operation {
    Pack,
    Unpack,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("failed to read `{path}`: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write `{path}`: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("output `{0}` already exists; pass --force to replace it")]
    OutputExists(PathBuf),

    #[error("provide a game with --game (preferred), or a password with --password")]
    PasswordSource,

    #[error(transparent)]
    Format(#[from] monke::Error),
}

fn main() {
    let cli = Cli::parse();
    let command_name = cli.command.name();

    if let Err(error) = run(cli) {
        render_error(command_name, error).exit();
    }
}

fn render_error(command_name: &str, error: CliError) -> clap::Error {
    let mut command = Cli::command();
    command.build();

    if let Some(subcommand) = command.find_subcommand_mut(command_name) {
        return subcommand.error(ErrorKind::Io, error);
    }

    command.error(ErrorKind::Io, error)
}

fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Unpack(args) => {
            let password = args.password(Operation::Unpack)?;
            let save = read_save(&args.input)?;
            let contents = monke::unpack(&save, password)?;
            write_file(&args.output, &contents, args.force)
        }
        Command::Pack { args, template } => {
            let password = args.password(Operation::Pack)?;
            let contents = read_file(&args.input)?;
            let template = template.as_deref().map(read_save).transpose()?;
            let save = monke::pack(&contents, password, template.as_ref())?;
            write_file(&args.output, &save.to_bytes()?, args.force)
        }
        Command::Inspect { input } => {
            let save = read_save(&input)?;
            print_save(&save);
            Ok(())
        }
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>, CliError> {
    fs::read(path).map_err(|source| CliError::Read {
        path: path.to_owned(),
        source,
    })
}

fn read_save(path: &Path) -> Result<SaveFile, CliError> {
    Ok(SaveFile::parse(&read_file(path)?)?)
}

fn write_file(path: &Path, contents: &[u8], force: bool) -> Result<(), CliError> {
    if !force && path.exists() {
        return Err(CliError::OutputExists(path.to_owned()));
    }

    // we write to a temporary file first to avoid corrupting the output if an error occurs
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = NamedTempFile::new_in(directory).map_err(|source| CliError::Write {
        path: path.to_owned(),
        source,
    })?;
    temporary
        .write_all(contents)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|source| CliError::Write {
            path: path.to_owned(),
            source,
        })?;

    let result = if force {
        temporary.persist(path).map(|_| ())
    } else {
        temporary.persist_noclobber(path).map(|_| ())
    };
    result.map_err(|error| CliError::Write {
        path: path.to_owned(),
        source: error.error,
    })
}

fn print_save(save: &SaveFile) {
    println!("header version: {}", save.header.version);
    println!("header length: {}", save.header.encoded_length());
    println!("save count: {}", save.header.save_count);
    println!("guid: {}", Hex(&save.header.guid));
    println!("unknown: {}", Hex(&save.header.unknown));
    println!("timestamp: 0x{:016x}", save.header.timestamp);
    println!("extension: {}", Hex(&save.header.extension));
    println!("password index: {}", save.password_index);
    println!("salt: {}", Hex(&save.salt));
    println!("ciphertext length: {}", save.ciphertext.len());
}

struct Hex<'a>(&'a [u8]);

impl fmt::Display for Hex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_short_aliases() {
        let cli = Cli::try_parse_from(["monke", "u", "input", "-o", "output", "--password", "11"])
            .unwrap();
        assert!(matches!(cli.command, Command::Unpack(_)));
    }

    #[test]
    fn parses_supported_game() {
        let cli =
            Cli::try_parse_from(["monke", "unpack", "input", "-o", "output", "--game", "btd6"])
                .unwrap();
        let Command::Unpack(args) = cli.command else {
            panic!("expected unpack command");
        };
        assert_eq!(args.password(Operation::Unpack).unwrap(), "11");
    }

    #[test]
    fn battd_uses_operation_specific_passwords() {
        let args = TransformArgs {
            input: "input".into(),
            output: "output".into(),
            game: Some(Game::Battd),
            password: None,
            force: false,
        };

        assert_eq!(
            args.password(Operation::Unpack).unwrap(),
            "$LXvKp90n$Cc0GTX2nY5"
        );
        assert_eq!(args.password(Operation::Pack).unwrap(), "16");
    }

    #[test]
    fn rejects_multiple_password_sources() {
        assert!(
            Cli::try_parse_from([
                "monke",
                "unpack",
                "input",
                "-o",
                "output",
                "--game",
                "btd6",
                "--password",
                "11",
            ])
            .is_err()
        );
    }
}
