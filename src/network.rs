use crate::{
    framebuffer::FrameBuffer,
    parser::{parse_pixelflut_commands, ParserState},
};
use log::{debug, info};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

const NETWORK_BUFFER_SIZE: usize = 256_000;

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

pub async fn handle_connection(
    mut stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
    ip: IpAddr,
    fb: Arc<FrameBuffer>,
) {
    debug!("Handling connection from {ip}");
    let mut buffer = [0u8; NETWORK_BUFFER_SIZE];
    // Number bytes left over **on the first bytes of the buffer** from the previous loop iteration
    let mut leftover_bytes_in_buffer = 0;

    // We have to keep the some things - such as connection offset - for the whole connection lifetime, so let's define them here
    let mut parser_state = ParserState::default();

    loop {
        // Fill the buffer up with new data from the socket
        // If there are any bytes left over from the previous loop iteration leave them as is and but the new data behind
        let bytes_read = match stream.read(&mut buffer[leftover_bytes_in_buffer..]).await {
            Ok(bytes_read) => bytes_read,
            Err(_) => {
                // statistics.dec_connections(ip);
                break;
            }
        };

        // statistics.inc_bytes(ip, bytes as u64);

        let data_end = leftover_bytes_in_buffer + bytes_read;
        if bytes_read == 0 {
            if leftover_bytes_in_buffer == 0 {
                // We read no data and the previous loop did consume all data
                // Nothing to do here, closing connection
                // statistics.dec_connections(ip);
                break;
            }

            // No new data from socket, read to the end and everything should be fine
            leftover_bytes_in_buffer = 0;
        } else {
            // Read some data, process it
            parser_state =
                parse_pixelflut_commands(&buffer[..data_end], &fb, &mut stream, parser_state).await;
            leftover_bytes_in_buffer = data_end - parser_state.last_byte_parsed;
        }

        if leftover_bytes_in_buffer > 0 {
            // We need to move the leftover bytes to the beginning of the buffer so that the next loop iteration con work on them
            buffer.copy_within(data_end - leftover_bytes_in_buffer..data_end, 0);
        }
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
