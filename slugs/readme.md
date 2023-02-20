# slugs
**s**mall, **l**ittle **g**id builder. (but with a u before the l an an s after the g so it's slugs 🥺)

usage: `slugs <directory>`

where directory looks something like this:
directory  
├ manifest.conf  
├ palette
├ frame1.tep  
├ frame2.tep  
├ frame3.tep

and manifest.conf looking like:
```
Width 8
Height 16
Palette palette
Delay 1

Frame frame1.tep
Frame frame2.tep
	Delay 2
Frame frame3.tep
```

```kdl
gif width=16 height=16 delay=4 {
	palette "palette"
	image "frame1.tep"
	image "frame2.tep" delay=2
	image "frame3.tep"
}
```