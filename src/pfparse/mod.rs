use thiserror::Error;

#[derive(Debug, Default)]
pub enum CommandParseState {
    #[default]
    PreCheck,
    Bytes,
    Pixel,
    Offset,
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("buffer too short")]
    BufTooShort,

    #[error("unknown/unsupported command")]
    UnknownCommand,

    #[error("invalid command end (missing newline)")]
    InvalidCommandEnd,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Size,
    PixelGet { x: usize, y: usize },
    PixelSet { c: u32, x: usize, y: usize },
    Offset { x: usize, y: usize },
}

pub async fn parse_command(buffer: &[u8]) -> Result<(Vec<Command>, usize), CommandParseError> {
    // Initialize the state. The parser state will keep track of the state we
    // are currently in. The state represents a state-machine. The index `i`
    // keeps track of the current offset in the buffer.
    let mut state = CommandParseState::default();
    let mut i = 0;
    let mut last_byte_parsed = 0;

    // The parsed commands get stored in this vector.
    let mut commands = Vec::new();

    // Pre-calculate the length of the buffer beforehand. Using this, we avoid
    // retrieving the length over and over again in the loop.
    let len = buffer.len() - 1;

    let loop_end = buffer.len().saturating_sub(crate::parser::PARSER_LOOKAHEAD);
    loop {
        if i >= loop_end {
            break;
        }
        state = match state {
            CommandParseState::PreCheck => {
                // This is the stopping condition. When we reach this point, we
                // need to exit the loop as we reached th end of the buffer.
                if i >= len {
                    break;
                }

                // Otherwise continue matching the bytes to select the proper
                // command.
                CommandParseState::Bytes
            }
            CommandParseState::Bytes => match buffer[i..] {
                [b'P', b'X', b' ', ..] => {
                    i += 3;
                    CommandParseState::Pixel
                }
                [b'O', b'F', b'F', b'S', b'E', b'T', b' ', ..] => {
                    i += 7;
                    CommandParseState::Offset
                }
                [b'H', b'E', b'L', b'P', b'\n', ..] => {
                    i += 5;
                    last_byte_parsed = i - 1;
                    commands.push(Command::Help);
                    CommandParseState::PreCheck
                }
                [b'S', b'I', b'Z', b'E', b'\n', ..] => {
                    i += 5;
                    commands.push(Command::Size);
                    last_byte_parsed = i - 1;
                    CommandParseState::PreCheck
                }
                _ => {
                    i += 1;
                    CommandParseState::Bytes
                } // _ => {
                  //     dbg!(&buffer[i..i + 3]);
                  //     return Err(CommandParseError::UnknownCommand);
                  // }
            },
            CommandParseState::Pixel => {
                // Look for the X coordinate. We do this by looping over the
                // bytes until we encounter a whitespace, which separates the
                // X from the Y coordinate.
                let (x, o) = read_number_until_whitespace(buffer, i);
                i = o;

                // Let's do the same as above for the Y coordinate.
                let (y, o) = read_number_until_whitespace_or_newline(buffer, i);
                i = o;

                // Exit early if user requested pixel at (X, Y)
                if buffer[i] == b'\n' {
                    commands.push(Command::PixelGet { x, y });
                    state = CommandParseState::PreCheck;
                    i += 1;
                    continue;
                }

                // Skip whitespace
                i += 1;

                // let str = unsafe { std::str::from_utf8_unchecked(&buffer[i..i + 6]) };
                // let color = u32::from_str_radix(str, 16).unwrap_or_default(); // TODO handle this
                let color: u32 = (ASCII_HEXADECIMAL_VALUES[buffer[i - 1] as usize] as u32) << 20
                    | (ASCII_HEXADECIMAL_VALUES[buffer[i] as usize] as u32) << 16
                    | (ASCII_HEXADECIMAL_VALUES[buffer[i - 3] as usize] as u32) << 12
                    | (ASCII_HEXADECIMAL_VALUES[buffer[i - 2] as usize] as u32) << 8
                    | (ASCII_HEXADECIMAL_VALUES[buffer[i - 5] as usize] as u32) << 4
                    | (ASCII_HEXADECIMAL_VALUES[buffer[i - 4] as usize] as u32);
                // Skip over color to newline
                i += 8;

                // Verify the pixel commands ends with a newline
                if buffer[i] != b'\n' && buffer[i] != 0 {
                    return Err(CommandParseError::InvalidCommandEnd);
                }

                // Push command and skip newline
                commands.push(Command::PixelSet { c: color, x, y });
                i += 1;

                last_byte_parsed = i - 1;
                CommandParseState::PreCheck
            }
            CommandParseState::Offset => {
                // Read the X coordinate until we encounter a whitespace
                let (x, o) = read_number_until_whitespace(buffer, i);
                i = o;

                // Let's do the same as above for the Y coordinate.
                let (y, o) = read_number_until_whitespace_or_newline(buffer, i);
                i = o;

                last_byte_parsed = i - 1;
                commands.push(Command::Offset { x, y });
                CommandParseState::PreCheck
            }
        }
    }

    Ok((commands, last_byte_parsed))
}

fn read_number_until_whitespace_or_newline(buffer: &[u8], offset: usize) -> (usize, usize) {
    let mut i = offset;
    let mut n = 0;

    loop {
        if buffer[i] == b' ' || buffer[i] == b'\n' || buffer[i] == 0 {
            break;
        }

        n = 10 * n + (buffer[i] - b'0') as usize;
        i += 1;
    }

    (n, i)
}

fn read_number_until_whitespace(buffer: &[u8], offset: usize) -> (usize, usize) {
    let mut i = offset;
    let mut n = 0;

    loop {
        if buffer[i] == b' ' || buffer[i] == 0 {
            i += 1;
            break;
        }

        n = 10 * n + (buffer[i] - b'0') as usize;
        i += 1;
    }

    (n, i)
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[tokio::test]
//     async fn parse_help_command() {
//         let input = "HELP\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmd) => assert_eq!(cmd[0], Command::Help),
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_size_command() {
//         let input = "SIZE\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmd) => assert_eq!(cmd[0], Command::Size),
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_pixel_get_command() {
//         let input = "PX 10 10\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmd) => assert_eq!(cmd[0], Command::PixelGet { x: 10, y: 10 }),
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_pixel_set_command() {
//         let input = "PX 10 10 000000\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmd) => assert_eq!(cmd[0], Command::PixelSet { x: 10, y: 10, c: 0 }),
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_offset_command() {
//         let input = "OFFSET 10 10\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmd) => assert_eq!(cmd[0], Command::Offset { x: 10, y: 10 }),
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_multiple_simple_commands() {
//         let input = "HELP\nSIZE\nHELP\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmds) => {
//                 assert_eq!(cmds.len(), 3);
//                 assert_eq!(cmds[0], Command::Help);
//                 assert_eq!(cmds[1], Command::Size);
//                 assert_eq!(cmds[2], Command::Help);
//             }
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_multiple_pixel_get_commands() {
//         let input = "PX 10 10\nPX 20 20\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmds) => {
//                 assert_eq!(cmds.len(), 2);
//                 assert_eq!(cmds[0], Command::PixelGet { x: 10, y: 10 });
//                 assert_eq!(cmds[1], Command::PixelGet { x: 20, y: 20 });
//             }
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_multiple_pixel_set_commands() {
//         let input = "PX 10 10 000000\nPX 20 20 000000\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmds) => {
//                 assert_eq!(cmds.len(), 2);
//                 assert_eq!(cmds[0], Command::PixelSet { x: 10, y: 10, c: 0 });
//                 assert_eq!(cmds[1], Command::PixelSet { x: 20, y: 20, c: 0 });
//             }
//             Err(err) => panic!("{err:?}"),
//         }
//     }

//     #[tokio::test]
//     async fn parse_multiple_pixel_commands() {
//         let input = "PX 10 10\nPX 20 20 000000\nPX 30 30\n";
//         let input = input.as_bytes();

//         match parse_command(input).await {
//             Ok(cmds) => {
//                 assert_eq!(cmds.len(), 3);
//                 assert_eq!(cmds[0], Command::PixelGet { x: 10, y: 10 });
//                 assert_eq!(cmds[1], Command::PixelSet { x: 20, y: 20, c: 0 });
//                 assert_eq!(cmds[2], Command::PixelGet { x: 30, y: 30 });
//             }
//             Err(err) => panic!("{err:?}"),
//         }
//     }
// }

const ASCII_HEXADECIMAL_VALUES: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0,
    0, 10, 11, 12, 13, 14, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 10, 11, 12, 13, 14, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];
