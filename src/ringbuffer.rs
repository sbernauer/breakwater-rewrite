// use crate::network::NETWORK_RING_BUFFER_SIZE;

// pub struct RingBuffer<'a> {
//     data: [u8; NETWORK_RING_BUFFER_SIZE],
// }

// impl<'a> RingBuffer<'a> {
//     pub fn new() -> Self {
//         Self {
//             data: [0u8; NETWORK_RING_BUFFER_SIZE],
//         }
//     }

//     pub fn tcp_stream_buffer(&self) -> &'a mut [u8] {
//         &mut self.data
//     }
// }
