# Audio Extension DRAFT
Add an MP3 audio stream to a gif. 

An application extension with the identifier "GENNYBLE" and auth code "AUD". The data is simply MP3 frames.

Questions yet answered:
- what do we do if the animation and audio length differ?
- what if there is no graphics extension and thus no length? do we behave differently?
- what if audio data starts before image data? do we play audio before we display?

What I'd like to do is just say "all we're doing is shoving MP3 frames in the extension, the rest is on you" and like, the decoder is just supposed to buffer and play the audio when it's received, but that seems.. not great.