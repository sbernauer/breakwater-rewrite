use std::sync::Arc;

use tokio::io::AsyncWriteExt;

use crate::{framebuffer::FrameBuffer, network::ClientState};

pub const PARSER_LOOKAHEAD: usize = "PX 1234 1234 rrggbbaa\n".len(); // Longest possible command
const HELP_TEXT: &[u8] = "\
Pixelflut server powered by breakwater https://github.com/sbernauer/breakwater
Available commands:
HELP: Show this help
PX x y rrggbb: Color the pixel (x,y) with the given hexadecimal color
PX x y rrggbbaa: Color the pixel (x,y) with the given hexadecimal color rrggbb (alpha channel is ignored for now)
PX x y: Get the color value of the pixel (x,y)
SIZE: Get the size of the drawing surface, e.g. `SIZE 1920 1080`
OFFSET x y: Apply offset (x,y) to all further pixel draws on this connection
".as_bytes();

#[derive(Default)]
pub struct ParserState {
    pub last_byte_parsed: usize,
}

/// Returns the offset (think of index in [u8]) of the laST bytes of the last fully parsed command.
pub async fn parse_pixelflut_commands(
    buffer: &[u8],
    fb: &Arc<FrameBuffer>,
    mut stream: impl AsyncWriteExt + Unpin,
    client_state: &mut ClientState,
    parser_state: &mut ParserState,
) {
    let mut x: usize;
    let mut y: usize;

    let mut i = 0; // We can't use a for loop here because Rust don't lets use skip characters by incrementing i
    while i + PARSER_LOOKAHEAD < buffer.len() {
        // FIXME
        parser_state.last_byte_parsed = i;
        if buffer[i] == b'P' {
            i += 1;
            if buffer[i] == b'X' {
                i += 1;
                if buffer[i] == b' ' {
                    i += 1;
                    // Parse first x coordinate char
                    if buffer[i] >= b'0' && buffer[i] <= b'9' {
                        x = (buffer[i] - b'0') as usize;
                        i += 1;

                        // Parse optional second x coordinate char
                        if buffer[i] >= b'0' && buffer[i] <= b'9' {
                            x = 10 * x + (buffer[i] - b'0') as usize;
                            i += 1;

                            // Parse optional third x coordinate char
                            if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                x = 10 * x + (buffer[i] - b'0') as usize;
                                i += 1;

                                // Parse optional forth x coordinate char
                                if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                    x = 10 * x + (buffer[i] - b'0') as usize;
                                    i += 1;
                                }
                            }
                        }

                        // Separator between x and y
                        if buffer[i] == b' ' {
                            i += 1;

                            // Parse first y coordinate char
                            if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                y = (buffer[i] - b'0') as usize;
                                i += 1;

                                // Parse optional second y coordinate char
                                if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                    y = 10 * y + (buffer[i] - b'0') as usize;
                                    i += 1;

                                    // Parse optional third y coordinate char
                                    if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                        y = 10 * y + (buffer[i] - b'0') as usize;
                                        i += 1;

                                        // Parse optional forth y coordinate char
                                        if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                            y = 10 * y + (buffer[i] - b'0') as usize;
                                            i += 1;
                                        }
                                    }
                                }

                                x += client_state.connection_x_offset;
                                y += client_state.connection_y_offset;

                                // Separator between coordinates and color
                                if buffer[i] == b' ' {
                                    i += 1;

                                    // Must be followed by 6 bytes RGB and newline or ...
                                    if buffer[i + 6] == b'\n' {
                                        i += 7; // We can advance one byte more than normal as we use continue and therefore not get incremented at the end of the loop

                                        let rgba: u32 = (from_hex_char(buffer[i - 3]) as u32) << 20
                                            | (from_hex_char(buffer[i - 2]) as u32) << 16
                                            | (from_hex_char(buffer[i - 5]) as u32) << 12
                                            | (from_hex_char(buffer[i - 4]) as u32) << 8
                                            | (from_hex_char(buffer[i - 7]) as u32) << 4
                                            | (from_hex_char(buffer[i - 6]) as u32);

                                        fb.set(x, y, rgba);
                                        if cfg!(feature = "count_pixels") {
                                            // statistics.inc_pixels(ip);
                                        }
                                        continue;
                                    }

                                    // ... or must be followed by 8 bytes RGBA and newline
                                    if buffer[i + 8] == b'\n' {
                                        i += 9; // We can advance one byte more than normal as we use continue and therefore not get incremented at the end of the loop

                                        let rgba: u32 = (from_hex_char(buffer[i - 5]) as u32) << 20
                                            | (from_hex_char(buffer[i - 4]) as u32) << 16
                                            | (from_hex_char(buffer[i - 7]) as u32) << 12
                                            | (from_hex_char(buffer[i - 6]) as u32) << 8
                                            | (from_hex_char(buffer[i - 9]) as u32) << 4
                                            | (from_hex_char(buffer[i - 8]) as u32);

                                        fb.set(x, y, rgba);
                                        if cfg!(feature = "count_pixels") {
                                            // statistics.inc_pixels(ip);
                                        }

                                        continue;
                                    }
                                }

                                // End of command to read Pixel value
                                if buffer[i] == b'\n' && x < fb.width && y < fb.height {
                                    if let Some(rgb) = fb.get(x, y) {
                                        match stream
                                            .write_all(
                                                format!(
                                                    "PX {} {} {:06x}\n",
                                                    // We don't want to return the actual (absolute) coordinates, the client should also get the result offseted
                                                    x - client_state.connection_x_offset,
                                                    y - client_state.connection_x_offset,
                                                    rgb.to_be() >> 8
                                                )
                                                .as_bytes(),
                                            )
                                            .await
                                        {
                                            Ok(_) => (),
                                            Err(_) => continue,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if buffer[i] == b'S' {
            i += 1;
            if buffer[i] == b'I' {
                i += 1;
                if buffer[i] == b'Z' {
                    i += 1;
                    if buffer[i] == b'E' {
                        stream
                            .write_all(format!("SIZE {} {}\n", fb.width, fb.height).as_bytes())
                            .await
                            .expect("Failed to write bytes to tcp socket");
                    }
                }
            }
        } else if buffer[i] == b'H' {
            i += 1;
            if buffer[i] == b'E' {
                i += 1;
                if buffer[i] == b'L' {
                    i += 1;
                    if buffer[i] == b'P' {
                        stream
                            .write_all(HELP_TEXT)
                            .await
                            .expect("Failed to write bytes to tcp socket");
                    }
                }
            }
        } else if buffer[i] == b'O'
            && buffer[i + 1] == b'F'
            && buffer[i + 2] == b'F'
            && buffer[i + 3] == b'S'
            && buffer[i + 4] == b'E'
            && buffer[i + 5] == b'T'
        {
            i += 6;
            if buffer[i] == b' ' {
                i += 1;
                // Parse first x coordinate char
                if buffer[i] >= b'0' && buffer[i] <= b'9' {
                    x = (buffer[i] - b'0') as usize;
                    i += 1;

                    // Parse optional second x coordinate char
                    if buffer[i] >= b'0' && buffer[i] <= b'9' {
                        x = 10 * x + (buffer[i] - b'0') as usize;
                        i += 1;

                        // Parse optional third x coordinate char
                        if buffer[i] >= b'0' && buffer[i] <= b'9' {
                            x = 10 * x + (buffer[i] - b'0') as usize;
                            i += 1;

                            // Parse optional forth x coordinate char
                            if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                x = 10 * x + (buffer[i] - b'0') as usize;
                                i += 1;
                            }
                        }
                    }

                    // Separator between x and y
                    if buffer[i] == b' ' {
                        i += 1;

                        // Parse first y coordinate char
                        if buffer[i] >= b'0' && buffer[i] <= b'9' {
                            y = (buffer[i] - b'0') as usize;
                            i += 1;

                            // Parse optional second y coordinate char
                            if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                y = 10 * y + (buffer[i] - b'0') as usize;
                                i += 1;

                                // Parse optional third y coordinate char
                                if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                    y = 10 * y + (buffer[i] - b'0') as usize;
                                    i += 1;

                                    // Parse optional forth y coordinate char
                                    if buffer[i] >= b'0' && buffer[i] <= b'9' {
                                        y = 10 * y + (buffer[i] - b'0') as usize;
                                        i += 1;
                                    }
                                }
                            }

                            // End of command to set offset
                            if buffer[i] == b'\n' {
                                client_state.connection_x_offset = x;
                                client_state.connection_y_offset = y;
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }
}

#[inline(always)]
fn from_hex_char(char: u8) -> u8 {
    match char {
        b'0'..=b'9' => char - b'0',
        b'a'..=b'f' => char - b'a' + 10,
        b'A'..=b'F' => char - b'A' + 10,
        _ => 0,
    }
}
