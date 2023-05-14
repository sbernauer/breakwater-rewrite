use crate::framebuffer::FrameBuffer;
use circbuf::CircBuf;
use log::{debug, info};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

// CircBuf::with_capacity(256_000) rounds to this number
pub const NETWORK_RING_BUFFER_SIZE: usize = 262_143;
// It's very important to not simply divide by a whole number!
// In that case the read from the TCP socket can stall because of the way we currently read using
// `.read(buffer.get_avail_upto_size(NETWORK_BATCH_SIZE)[0])`
const NETWORK_BATCH_SIZE: usize = NETWORK_RING_BUFFER_SIZE / 2 + 1;

pub struct Network {
    listen_address: String,
    fb: Arc<FrameBuffer>,
}

impl Network {
    pub fn new(listen_address: String, fb: Arc<FrameBuffer>) -> Self {
        Network { listen_address, fb }
    }

    pub async fn listen(&self) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(&self.listen_address).await?;
        info!("Started Pixelflut server on {}", self.listen_address);

        loop {
            let (socket, socket_addr) = listener.accept().await?;
            // If you connect via IPv4 you often show up as embedded inside an IPv6 address
            // Extracting the embedded information here, so we get the real (TM) address
            let ip = ip_to_canonical(socket_addr.ip());

            let fb_for_thread = Arc::clone(&self.fb);
            tokio::spawn(async move {
                handle_connection(socket, ip, fb_for_thread).await;
            });
        }
    }
}

#[derive(Default)]
pub struct ClientState {
    pub connection_x_offset: usize,
    pub connection_y_offset: usize,
}

pub async fn handle_connection(
    // mut stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
    mut stream: TcpStream,
    ip: IpAddr,
    _fb: Arc<FrameBuffer>,
) {
    debug!("Handling connection from {ip}");
    let mut buffer =
        CircBuf::with_capacity(NETWORK_RING_BUFFER_SIZE).expect("Failed to create ringbuffer");
    debug!("Crated ringbuffer of size {}", buffer.cap());

    // let mut buffer = [0u8; NETWORK_BUFFER_SIZE];
    // // Number of bytes left over **on the first bytes of the buffer** from the previous loop iteration
    // let mut leftover_bytes_in_buffer = 0;

    // We have to keep the some things - such as connection offset - for the whole connection lifetime, so let's define them here
    // let mut client_state: ClientState = ClientState::default();
    // let mut parser_state: ParserState = ParserState::default();

    loop {
        // Fill the ringbuffer up with new data from the socket
        let bytes_read = match stream
            .read(buffer.get_avail_upto_size(NETWORK_BATCH_SIZE)[0])
            .await
        {
            // let bytes_read = match stream.readv(&buffer.get_avail()) { // Had bad performance
            Ok(bytes_read) => bytes_read,
            Err(_) => {
                // statistics.dec_connections(ip);
                break;
            }
        };
        buffer.advance_write(bytes_read);

        let buffer_byte_slices = &buffer.get_bytes();
        let read_right = buffer_byte_slices[0];
        let read_left = buffer_byte_slices[1];
        // let read = read_right.iter().chain(read_left.iter());
        let read_len = read_right.len() + read_left.len();

        // if read_len < PARSER_LOOKAHEAD {
        //     continue; // Try to get more data
        // }

        let mut x = 0;

        // for i in read {
        //     // x = *i;
        // }
        if read_left.len() != 0 {
            dbg!(read_right.len(), read_left.len());
        }
        for i in read_right {
            x = *i;
        }
        for i in read_left {
            x = *i;
        }
        buffer.advance_read(read_len);

        // if bytes_read != read_len {
        //     dbg!(bytes_read, read_len);
        // }
        // statistics.inc_bytes(ip, bytes as u64);

        // loop {
        //     let next = match buffer.get() {
        //         Ok(next) => next,
        //         Err(CircBufError::BufEmpty) => {
        //             // yield_now().await;
        //             break;
        //         }
        //         Err(_) => {
        //             panic!("TODO, this should not happen")
        //         }
        //     };
        // }

        // parse_pixelflut_commands(
        //     &buffer[..data_end + PARSER_LOOKAHEAD],
        //     &fb,
        //     &mut stream,
        //     &mut client_state,
        //     &mut parser_state,
        // )
        // .await;

        // // Fill the buffer up with new data from the socket
        // // If there are any bytes left over from the previous loop iteration leave them as is and put the new data behind
        // // We also need to leave `PARSER_LOOKAHEAD` space at the end, so the parser does not need to check boundaries all over the place
        // let bytes_read = match stream
        //     .read(&mut buffer[leftover_bytes_in_buffer..NETWORK_BUFFER_SIZE - PARSER_LOOKAHEAD])
        //     .await
        // {
        //     Ok(bytes_read) => bytes_read,
        //     Err(_) => {
        //         // statistics.dec_connections(ip);
        //         break;
        //     }
        // };

        // statistics.inc_bytes(ip, bytes as u64);

        // let mut data_end = leftover_bytes_in_buffer + bytes_read;
        // assert!(data_end <= NETWORK_BUFFER_SIZE);
        // assert!(data_end <= NETWORK_BUFFER_SIZE - PARSER_LOOKAHEAD);

        // if bytes_read == 0 {
        //     if leftover_bytes_in_buffer == 0 {
        //         // We read no data and the previous loop did consume all data
        //         // Nothing to do here, closing connection
        //         // statistics.dec_connections(ip);
        //         break;
        //     }

        //     // No new data from socket, read to the end and everything should be fine
        //     // Next loop iteration will likely close connection
        //     leftover_bytes_in_buffer = 0;
        // } else {
        //     // We have read some data, process it

        //     // We need to zero the PARSER_LOOKAHEAD bytes, so the parser does not detect any command left over from a previous loop iteration
        //     for i in &mut buffer[data_end..data_end + PARSER_LOOKAHEAD] {
        //         *i = 0;
        //     }

        //     parse_pixelflut_commands(
        //         &buffer[..data_end + PARSER_LOOKAHEAD],
        //         &fb,
        //         &mut stream,
        //         &mut client_state,
        //         &mut parser_state,
        //     )
        //     .await;

        //     leftover_bytes_in_buffer = data_end - parser_state.last_byte_parsed;
        // }

        // if leftover_bytes_in_buffer > 0 {
        //     // We need to move the leftover bytes to the beginning of the buffer so that the next loop iteration can work on them
        //     buffer.copy_within(data_end - leftover_bytes_in_buffer..data_end, 0);
        // }
    }
}

/// TODO: Switch to official ip.to_canonical() method when it is stable. **If** it gets stable sometime ;)
/// See <https://doc.rust-lang.org/std/net/enum.IpAddr.html#method.to_canonical>
fn ip_to_canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(v6) => match v6.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, a, b, c, d] => {
                IpAddr::V4(Ipv4Addr::new(a, b, c, d))
            }
            _ => ip,
        },
    }
}
