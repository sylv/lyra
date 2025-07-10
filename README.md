# lyra

## ideas

- Option to move files between backends
  - If I start watching an episode, automatically download the entire episode and move it to the local fs
  - If I watch S01E01, automatically cache S01E02 or the rest of the season
  - Run intro/outro extraction, thumbnail generation etc once local
  - Automatically move local stuff to a remote mount (or delete it if from a read-only mount) to make room
- Chapter extraction on request
- On-demand whisper subtitles
  - WebVTT with HLS is segmented, so we can abuse that

## notes

- HLS implementation
  - Not even close to spec compliant, but seems to work
  - Need a better way to test segment splitting without running the entire thing