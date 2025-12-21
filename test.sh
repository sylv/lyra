ffmpeg -i test.mkv -map 0:0 -c copy \
        -movflags +empty_moov+default_base_moof+omit_tfhd_offset \
        -f mp4 -t 0 ./segments/init.mp4
            
ffmpeg -i test.mkv -map 0:0 -c copy \
         -f stream_segment \
         -segment_times 6,12,18 \
         -segment_format mp4 \
         -segment_format_options "movflags=+frag_keyframe+separate_moof+omit_tfhd_offset+default_base_moof" \
         -segment_time_delta 0.05 \
         -reset_timestamps 1 \
         ./segments/seg_%d.m4s