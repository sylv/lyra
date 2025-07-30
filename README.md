# lyra

## ideas

- Option to move files between backends
  - If I start watching an episode, automatically download the entire episode and move it to the local fs
  - If I watch S01E01, automatically cache S01E02 or the rest of the season
  - Run intro/outro extraction, thumbnail generation etc once local
  - Automatically move local stuff to a remote mount (or delete it if from a read-only mount) to make room
- Include IMDb IDs in database because they're universal and will allow for more metadata providers in the future
- Download images on match instead of on request
- Share media items (shows, seasons, etc) with users through one-time links without needing an account
- Support matching different episode orderings
  - Not sure how to do this yet, with episode names it would be possible but otherwise it might be hard
  - If there are multiple complete seasons on disk we can use that maybe?
- On-demand whisper subtitles
  - WebVTT with HLS is segmented, so we can abuse that

## developer notes

- HLS implementation
  - Not even close to spec compliant, but seems to work
  - Need a better way to test segment splitting without running the entire thing
- The entire GraphQL API requires auth
  - During setup, there is a special init endpoint that returns a setup token if no users exist yet
  - Logins are handled by raw JSON that bypasses the auth
  - Every other endpoint requires authentication

## todo

- Timeline previews for local files
- Use CLIP to encode screenshots every 5 seconds and then support reverse search for scenes (and searching by scene contents?)
- ffmpeg segmenter is inconsistent, seeking forward then back will cause the player to freeze/skip a few seconds
  - i dont think this is a keyframe issue
  - possibly related to how seeking is done and it being offset, especially toward the end of the file, because the hls muxer can split += 0.01s from the intended position
- Subtitle support
- Chapter support
- Support for hardware transcoding
- Homepage with keep watching, recommendations, etc
- Cache ffprobe results
- Extract codec tag for hls.js to use
- Support playing different editions properly
- Support for other backends (WebDAV, S3 to start)
- Split reader/writer pool to avoid `DATABASE LOCKED` errors?
- Thumbhash images + store height/width to prevent layout shifts
  - Blocked because sqlx provides no good way to load nested structs/relations
  - sea-orm would work but not sure how relations are serialized, probably needs juno changes
  - Images should pick a random gradient at first, then transition to the thumbhash once loaded, then transition to the actual image with crossfade
  - Can probably only be done once switched to GraphQL
- TMDb attribution
- Segmenters are never cleaned up
- Scanner improvements
  - Batch file inserts
  - Use concurrent folder scanning to improve network performance
- Setup needs lots of work
  - Errors are not shown properly
  - It abruptly closes the modal once setup is complete, instead showing some tips/where to find settigns would be good
  - Needs a library setup step and for libraries to be moved to db
  - There is no way to add new users
  - There is no way to use invite code
  - There is no way to create invites
- Ffmpeg sometimes crashes and does not recover
- The segmenters `JUMP_SIZE` and `BUFFER_SIZE` options should be determined by how long segments take to generate
- Increasing the length of subtitle segments would make sense
- "fullscreen_on_play" persisted user option that determines if clicking play auto-fullscreens the player
- Easy way to import watch states from plex and maybe jellyfin
- Easy way to import/export watch states from/to a csv
- Search should find episodes by name