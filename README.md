# gifed
Gifed is a **GIF** **e**ncoding and **d**ecoding library.

At current, half of that statement is a lie and this crate is incredibly rough. This library can
only *write* gifs. The project is in its infancy but I hope to make it feature complete and
intuitive to use while still allowing fine grained control over the file itself, which has been and 
always will be, one of the main goals of this crate.

### TODO
- [x] Writing GIF87a
- [x] Writing GIF89a
- [x] Automatically select the lowest version possible when writing
- [ ] Read GIF87a
- [ ] Read GIF89a
- [ ] Feature to allow using the [weezl][weezl-crates] crate for LZW compression instead of the built-in
- [ ] Feature to allow using the [rgb][rgb-crates] crate for the color type.
- [ ] Well written and easy to understand docs! `bitvec` quality, but who can match that?

[weezl-crates]: https://crates.io/crates/weezl
[rgb-crates]: https://crates.io/crates/rgb