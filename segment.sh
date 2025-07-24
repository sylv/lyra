# ffmpeg -y -i test/video_2.mkv \
#   -copyts -sn -an -c:0 copy -start_at_zero -avoid_negative_ts disabled \
#   -f segment -break_non_keyframes 1 -segment_time 10 -segment_format mpegts -segment_list segments/index.m3u8 segments/seg-%05d.ts
rm -rf segments/*
ffmpeg -y -i "$1" -copyts -sn -an -c:0 copy -start_at_zero -avoid_negative_ts disabled \
  -f hls -hls_flags +split_by_time+temp_file -hls_segment_type mpegts -hls_time 10 -hls_list_size 0 -hls_segment_filename segments/seg-%d.ts -