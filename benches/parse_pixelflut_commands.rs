use breakwater::{
    framebuffer::FrameBuffer,
    parser::{parse_pixelflut_commands, ParserState},
    test::helpers::DevNullTcpStream,
};
use criterion::{
    BenchmarkId, Criterion, {criterion_group, criterion_main},
};
use std::sync::Arc;

async fn invoke_parse_pixelflut_commands(
    input: &[u8],
    fb: &Arc<FrameBuffer>,
    parser_state: &ParserState,
) {
    let mut stream = DevNullTcpStream::default();
    parse_pixelflut_commands(input, fb, &mut stream, parser_state.clone()).await;
}

fn from_elem(c: &mut Criterion) {
    let input = [0_u8; 100000];
    let fb = Arc::new(FrameBuffer::new(1920, 1080));
    let parser_state = ParserState::default();

    c.bench_with_input(
        BenchmarkId::new("parse_pixelflut_commands", "default"),
        &(&input, &fb, &parser_state),
        |b, s| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| invoke_parse_pixelflut_commands(s.0, s.1, s.2));
        },
    );
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
