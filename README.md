# gifed
Gifed is a **GIF** **e**ncoding and **d**ecoding library.

This crate is still pretty rough. I hope to make it feature complete and intuitive to use while still allowing fine grained control over the file itself, which has been and always will be, one of the main goals of this crate.

### Gifed TODO
- [x] Writing GIF87a
- [x] Writing GIF89a
- [x] Automatically select the lowest version possible when writing
- [x] Read GIF87a
- [x] Read GIF89a
- [ ] Reader
- [x] Streaming Reader
- [x] Writer
- [ ] Streaming Writer
- [ ] Feature to allow using the [weezl][weezl-crates] crate for LZW compression instead of the built-in
- [ ] Feature to allow using the [rgb][rgb-crates] crate for the color type.
- [ ] Well written and easy to understand docs! `bitvec` quality, but who can match that?

[weezl-crates]: https://crates.io/crates/weezl
[rgb-crates]: https://crates.io/crates/rgb

#### Extensions
These are part of the 89a spec, but are kept separate as they're not "core" to the spec

- [x] Application Extension
- [x] Comment Extension
- [ ] Plain Text Extension
- [ ] Netscape Looping Extension ([details][netscape])

### Relevant Writings

- [gif87a][gif87a]
- [gif89a][gif89a]
- [Netscape Looping Extension][netscape]

[gif87a]: https://www.w3.org/Graphics/GIF/spec-gif87.txt
[gif89a]: https://www.w3.org/Graphics/GIF/spec-gif89a.txt
[netscape]: http://www.vurdalakov.net/misc/gif/netscape-looping-application-extension

## gifprobe
Similar to FFMPEG's ffprobe, gifprobe will print details of a gif to stdout.

## giftool
A CLI tool for modifying gif files.

# License
gifed, gifprobe, and giftool are licensed under Creative Commons Zero 1.0; they're in the public domain. Attribution is appreciated, but not required.