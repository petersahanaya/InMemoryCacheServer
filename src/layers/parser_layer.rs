use core::fmt;

#[derive(Debug)]
pub enum Command {
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
        ttl: Option<u64>,
    },
    SetX {
        key: String,
        value: String,
        ttl: u64,
    },
    Delete {
        key: String,
    },
    Help,
    Stats,
    Unknown,
}

#[derive(Debug)]
pub enum ParseErr {
    InvalidCommand(String),
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCommand(msg) => write!(f, "{msg}"),
        }
    }
}

impl Command {
    pub fn parse(raw_cmd: &str) -> Result<Self, ParseErr> {
        let cmd_parts: Vec<&str> = raw_cmd.trim().split_whitespace().collect();

        let command = cmd_parts[0];

        match command {
            "GET" => {
                if cmd_parts.len() < 2 {
                    return Err(ParseErr::InvalidCommand(
                        "Expect <COMMAND> <KEY>.".to_string(),
                    ));
                }

                let key = cmd_parts[1].to_string();

                Ok(Command::Get { key })
            }
            "SET" => {
                if cmd_parts.len() < 3 {
                    return Err(ParseErr::InvalidCommand(
                        "Expect <COMMAND> <KEY> <VALUE>.".to_string(),
                    ));
                };

                let key = cmd_parts[1].to_string();
                let value = cmd_parts[2].to_string();

                let ttl = if cmd_parts.len() >= 5 && cmd_parts[3].contains("EX") {
                    cmd_parts[4].parse::<u64>().ok()
                } else {
                    None
                };

                Ok(Command::Set { key, value, ttl })
            }
            "SETX" => {
                if cmd_parts.len() < 4 {
                    return Err(ParseErr::InvalidCommand(
                        "Expect <COMMAND> <KEY> <VALUE> <TTL>.".to_string(),
                    ));
                };

                let key = cmd_parts[1].to_string();
                let value = cmd_parts[2].to_string();
                let ttl = cmd_parts[3].parse::<u64>().map_err(|_| {
                    ParseErr::InvalidCommand("TTL must be a valid positive integer.".to_string())
                })?;

                Ok(Command::SetX { key, value, ttl })
            }
            "DELETE" | "DEL" => {
                if cmd_parts.len() < 2 {
                    return Err(ParseErr::InvalidCommand(
                        "Expect <COMMAND> <KEY>.".to_string(),
                    ));
                }

                let key = cmd_parts[1];

                Ok(Command::Delete {
                    key: key.to_string(),
                })
            }
            "HELP" => Ok(Command::Help),
            "STATS" => Ok(Command::Stats),
            _ => Ok(Command::Unknown),
        }
    }
}
