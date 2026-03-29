cid=$(docker create sylver/lyra-static-ffmpeg)
docker cp "$cid":/lyra-ffmpeg ./bin/lyra-ffmpeg
docker cp "$cid":/lyra-ffprobe ./bin/lyra-ffprobe
docker rm "$cid"
chmod +x ./bin/lyra-ffmpeg ./bin/lyra-ffprobe