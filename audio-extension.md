# Audio Extension DRAFT
Add an MP3 audio stream to a gif. 

An application extension with the identifier "GENNYBLE" and auth code "AUD".

Rough idea:
- we need an "Audio Control Extension", which is similar to a "Graphic Control Extension". It will provide detail on the upcoming audio data and where it appears so it may inform the decoder.
- two version:
	- one that's more era appropriate with MP3
	- one with Opus which is just cuter

## Audio Control Extension
Application Extension. Ident "GENNYBLE" auth code "ACE" *(audio control extension)*.

problems:
- a decoder may stop reading blocks after it draws an image that has a graphic control with delay. if there is supposed to be audio playing with this frame, it won't know.

## ahh
- a fixed timescale counting up from the first image every hundreth of a second. audio may not play first.

The stream is driven by the gif video and assumed to be in sync from when it starts.

for audio to be played, there **must** be an ACN extension before the image it's to be played with. this informs the decoder that it's to continue processing after it draws the image. directly after the image should appear the ADT extension

The gif image data drives the audio. The audio **must not** extend the time of
the file. 

Because the minimal length of an MP3 frame is 1152 samples *(something about size)* the buffer **must** be able to contain a frame of MP3 data. 

## Audio Data Block Extension
Application Extension. Ident "GENNYBLE" auth code "ADT" *(audio data)*.


## Example Data Stream
GCE - delay 0.1
ACE - audio after image
IMG - image
ADT - audio, dur 0.09, delay 0.01