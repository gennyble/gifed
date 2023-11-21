# gifed
Gifed is a **GIF** **e**ncoding and **d**ecoding library.

This crate is getting somewhere! I hope to polish the API a touch and introduce an API for quantization.

### Gifed TODO
- [x] Feature to allow using the [weezl][weezl-crates] crate for LZW compression instead of the built-in. *(works with compression! we still require weezl for decompression)*
- [ ] Feature to allow using the [rgb][rgb-crates] crate for the color type.
- [ ] Well written and easy to understand docs! `bitvec` quality, but who can match that?

[weezl-crates]: https://crates.io/crates/weezl
[rgb-crates]: https://crates.io/crates/rgb

#### Extensions
These are part of the 89a spec, but are kept separate as they're not "core" to the spec

- [x] Application Extension
- [x] Comment Extension
- [ ] Plain Text Extension
- [x] Netscape Looping Extension ([details][netscape])

### Relevant Writings

- [gif87a][gif87a]
- [gif89a][gif89a]
- [Netscape Looping Extension][netscape]

[gif87a]: https://www.w3.org/Graphics/GIF/spec-gif87.txt
[gif89a]: https://www.w3.org/Graphics/GIF/spec-gif89a.txt
[netscape]: http://www.vurdalakov.net/misc/gif/netscape-looping-application-extension

## gifprobe
Similar to FFMPEG's ffprobe, gifprobe will print details of a gif to stdout.

## gaudio
Nothing valued is here. Inject/Retrieve MP3 files from GiFs. I'd like to eventually play them right outta the thing, right, but that's a lot harder.

# License
gifed, gifprobe, gifcheck, and gaudio are licensed under ISC.
