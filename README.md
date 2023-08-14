# viewd
A image veiwer server and client for viewing images on a remote box (presumably with monitor connected). A learning project that filled a personal niche. Probably not for general use, though it is useful to me. It uses quic for network commnunication and sdl2 for image dispaly. 

Only tested on linux.

## usage

Accepts a directory containing images as input. It *should* filter out
non-image files. No recursion is done into sub-directories.

on the display box:
	viewd server --path ~/dir/photos/

on the client:
	viewd client

Commands are read from client stdin.

### commands

Currently supported commands are

	* `->` (arrow right) next image
    * `<-` (arrow left) previous image
	* `f`  fullscreen
	* `r`  rotate
	* `p`  pageant mode (automatically scroll through the images)
    * `q`  quit (the client)

## dependencies

You need sdl libraries on your OS. Milage may vary depending on sytem, but on debian-like apt can obtain them for you: 

	sudo apt-get install libsdl2-image-2.0-0

## display

You may need to export your display. `:1` may or may not be correct
depending on your system.

	export DISPLAY=:1

## development

You will need development libraries for builds to complete.

	sudo apt-get install libsdl2-image-dev
