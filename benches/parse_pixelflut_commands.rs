use breakwater::{
    framebuffer::FrameBuffer,
    parser::{parse_pixelflut_commands, ParserState},
    test::helpers::{get_commands_to_draw_rect, get_commands_to_read_rect, DevNullTcpStream},
};
use criterion::{
    BenchmarkId, Criterion, {criterion_group, criterion_main},
};
use std::sync::Arc;

const FRAMEBUFFER_WIDTH: usize = 1920;
const FRAMEBUFFER_HEIGHT: usize = 1080;

async fn invoke_parse_pixelflut_commands(
    input: &[u8],
    fb: &Arc<FrameBuffer>,
    parser_state: ParserState,
) {
    let mut stream = DevNullTcpStream::default();
    parse_pixelflut_commands(input, fb, &mut stream, parser_state).await;
}

fn from_elem(c: &mut Criterion) {
    let draw_commands = get_commands_to_draw_rect(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, 0x123456);
    let draw_commands = draw_commands.as_bytes();

    c.bench_with_input(
        BenchmarkId::new(
            "parse_draw_commands",
            format!("{FRAMEBUFFER_WIDTH} x {FRAMEBUFFER_HEIGHT}"),
        ),
        &draw_commands,
        |b, input| {
            let fb = Arc::new(FrameBuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT));
            let parser_state = ParserState::default();
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| invoke_parse_pixelflut_commands(input, &fb, parser_state.clone()));
        },
    );

    let read_commands = get_commands_to_read_rect(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
    let read_commands = read_commands.as_bytes();

    c.bench_with_input(
        BenchmarkId::new(
            "parse_read_commands",
            format!("{FRAMEBUFFER_WIDTH} x {FRAMEBUFFER_HEIGHT}"),
        ),
        &read_commands,
        |b, input| {
            let fb = Arc::new(FrameBuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT));
            let parser_state = ParserState::default();
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| invoke_parse_pixelflut_commands(input, &fb, parser_state.clone()));
        },
    );
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
