# play-file
A no frills port of Apple's PlayFile CoreAudio Sample to Rust

Makes use of a fork of coreaudio-sys and some code from coreaudio-rs

To Do: This is currently pinned to my fork of coreaudio-sys to get audio toolbox framework integration. Maybe merge that
back into coreaudio-sys? Hampered by the broken-ness of rust-bindgen on OSX of late.