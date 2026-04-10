use crate::{Codec, Stream, StreamDetails};

pub const TRANSCODED_H264_VIDEO_CODEC_TAG: &str = "avc1.42E01E";

pub fn video_codec_tag(stream: &Stream) -> Option<String> {
    match (&stream.codec, &stream.details) {
        (
            Codec::VideoH264,
            StreamDetails::Video {
                profile,
                level,
                codec_tag_string,
                ..
            },
        ) => Some(format!(
            "{}.{}{}{:02X}",
            h264_sample_entry(codec_tag_string.as_deref()),
            h264_profile_hex(profile.as_deref())?,
            h264_constraints_hex(profile.as_deref())?,
            h264_level_idc(*level)?
        )),
        (
            Codec::VideoH265,
            StreamDetails::Video {
                profile,
                level,
                bit_depth,
                codec_tag_string,
                ..
            },
        ) => Some(format!(
            "{}.{}.6.L{}.B0",
            h265_sample_entry(codec_tag_string.as_deref()),
            h265_profile_idc(profile.as_deref(), *bit_depth)?,
            h265_level_idc(*level)?
        )),
        (
            Codec::VideoAv1,
            StreamDetails::Video {
                profile,
                level,
                bit_depth,
                ..
            },
        ) => Some(format!(
            "av01.{}.{}M.{:02}",
            av1_profile_idc(profile.as_deref())?,
            av1_level_idc(*level)?,
            bit_depth.unwrap_or(8)
        )),
        _ => None,
    }
}

pub fn audio_codec_tag(codec: &Codec) -> Option<&'static str> {
    match codec {
        Codec::AudioAac => Some("mp4a.40.2"),
        _ => None,
    }
}

fn h264_sample_entry(codec_tag_string: Option<&str>) -> &'static str {
    match codec_tag_string {
        Some("avc1") => "avc1",
        Some("avc3") => "avc3",
        _ => "avc1",
    }
}

fn h265_sample_entry(codec_tag_string: Option<&str>) -> &'static str {
    match codec_tag_string {
        Some("hvc1") => "hvc1",
        Some("hev1") => "hev1",
        _ => "hvc1",
    }
}

fn h264_profile_hex(profile: Option<&str>) -> Option<&'static str> {
    match profile?.to_ascii_lowercase().as_str() {
        "constrained baseline" => Some("42"),
        "baseline" => Some("42"),
        "main" => Some("4D"),
        "extended" => Some("58"),
        "high" => Some("64"),
        "high 10" => Some("6E"),
        "high 4:2:2" => Some("7A"),
        "high 4:4:4 predictive" => Some("F4"),
        _ => None,
    }
}

fn h264_constraints_hex(profile: Option<&str>) -> Option<&'static str> {
    match profile?.to_ascii_lowercase().as_str() {
        "constrained baseline" => Some("E0"),
        "baseline" => Some("C0"),
        "main" => Some("40"),
        "extended" => Some("00"),
        "high" => Some("00"),
        "high 10" => Some("00"),
        "high 4:2:2" => Some("00"),
        "high 4:4:4 predictive" => Some("00"),
        _ => None,
    }
}

fn h264_level_idc(level: Option<i32>) -> Option<u8> {
    u8::try_from(level?).ok()
}

fn h265_profile_idc(profile: Option<&str>, bit_depth: Option<u8>) -> Option<u8> {
    match profile?.to_ascii_lowercase().as_str() {
        "main" => Some(1),
        "main 10" => Some(2),
        "main still picture" => Some(3),
        _ if bit_depth == Some(10) => Some(2),
        _ => None,
    }
}

fn h265_level_idc(level: Option<i32>) -> Option<i32> {
    Some(level?)
}

fn av1_profile_idc(profile: Option<&str>) -> Option<u8> {
    match profile?.to_ascii_lowercase().as_str() {
        "main" => Some(0),
        "high" => Some(1),
        "professional" => Some(2),
        _ => None,
    }
}

fn av1_level_idc(level: Option<i32>) -> Option<String> {
    let level = level?;
    let major = level / 4;
    let minor = level % 4;
    Some(format!("{major}{minor}"))
}
