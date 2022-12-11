# Recolor Extension DRAFT

I want to assume that if the decoder does not know the type of an extension, IE the extension label does not match with graphic control/comment/plain text/application, that it will silently ignore it. The decoder can always know the size, so it seems plausbile. But I cannot find any indication of so in the spec for 87a or 89a.

## Extension Scope
The scope of this extension is every block following this one.

## Extension Type
### Regular Extension
In the case where we determine that we don't need to "wrap" it in an application extension this extension's label shall be `0xF8`. This positions it directly below the Graphic Control Extension, `0xF9`.

### Application Extension
In the case where we determine that we *should* "wrap" in an application extension this extensions identifier shal be "GENNYBLE" and the Authentication Code "REC".

## Extension Data
Regardless of whatever extension we're wrapped in, here's what the data will look like:

### Information Sub-block
This is a sub-block of length 2 and contained a packed byte followed by the background color index. The packed byte is specified to be the same as the Logical Screen Descriptors packed byte with the Global Color Table Flag as true. The details of the packed byte were copied below from the Logical Screen Descriptor section for convinience.

#### Packed Byte
Packed Fields:  
1 bit : Global Color Table Flag  
3 bits: Color Resolution  
1 bit : Sort Flag  
3 bits: Size of Global Color Table  

##### Global Color Table Flag
Contains the fixed value `1`

##### Color Resolution
Number of bits per primary color available
to the original image, minus 1. This value represents the size of
the entire palette from which the colors in the graphic were
selected, not the number of colors actually used in the graphic.
For example, if the value in this field is 3, then the palette of
the original image had 4 bits per primary color available to create
the image.  This value should be set to indicate the richness of
the original palette, even if not every color from the whole
palette is available on the source machine.

##### Sort Flag
Indicates whether the Global Color Table is sorted.
If the flag is set, the Global Color Table is sorted, in order of
decreasing importance. Typically, the order would be decreasing
frequency, with most frequent color first. This assists a decoder,
with fewer available colors, in choosing the best subset of colors;
the decoder may use an initial segment of the table to render the
graphic.

Values:  
0 - Not ordered.
1 - Ordered by decreasing importance, most important color first.

##### Size of Global Color Table
The value in this field is used to calculate the number
of bytes contained in the Global Color Table. To determine that
actual size of the color table, raise 2 to [the value of the field
+ 1]. (This field is made up of the 3 least significant bits of the byte.)

#### Background Color Index
Index into the Global Color Table for
the Background Color. The Background Color is the color used for
those pixels on the screen that are not covered by an image.

### Color Table
Contains a number of bytes equal to 3 x 2^(Size of Global Color Table+1).

The color table goes one color after the other, split into as many data sub-blocks as neccesary *(4, at most)*.