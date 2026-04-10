use crate::Stream;

pub fn video_display_info(stream: &Stream) -> String {
    let resolution = match (stream.width(), stream.height()) {
        (Some(width), Some(height)) => format!("{width}x{height}"),
        _ => "Unknown".to_string(),
    };
    let frame_rate = stream
        .frame_rate()
        .map(|rate| format!("@{rate:.0}"))
        .unwrap_or_default();
    let bitrate = stream
        .bit_rate
        .map(|bits| format!(" {}Mbps", bits / 1_000_000))
        .unwrap_or_default();
    format!("Original {resolution}{frame_rate}{bitrate}")
}
