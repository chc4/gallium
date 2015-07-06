A WM written in Rust, inspired and based loosely off of Kintaro's wtftw project.
This is a toy project. Don't even think about using it.

Brainstorming:
Each workspace will have a Layout, a deck of tiling windows, a deck of floating windows, and iconified windows.
Each deck has a "current" index, which is the selected window in the deck.

There should probably be core 5 Layouts - Tall, Wide, Grid, Focused, and Floating.
Tall tiles vertically, Wide horizontally, Grid will have a resizable number of rows and columns with an overflow square in the bottom-right, Focused will full-screen the currently selected window and hide the rest, and Floating will remap all existing windows and any new ones to floating, but switch them back when switched to another layout.

The bar will probably just be a plug-n-play bar, with the WM exporting the current WM, window selected, etc. Not sure if you can do some info bars would like without XCB though.

Config will be hotswappable, and probably the entire WM as well. The config will just be in JSON format since there is already a nice serializer for it in place.
Keybinds are in Emac's format since it's an easy way to say them. M4-r means "the windowss key+r". You can add as many modifiers as you want ("S-","Lock-","C-","M-","M2-","M3-","M4-","M5-" are the options, plus a "K-" modifier that stands for the config-defined prefix. Shift, Lock, Control, Meta(alt), Meta2-5, Kommand.) All keybinds will be defined this way, and reloadable from the config file.

Still to come:
* Better debug than just println! pls
* Switching workspaces and sending windows between them
* Floating window support
* Better serialization of colors, including referencing your terminal colors instead
* More layout implementations (including hopefully a BSP mode!)
* The ability to reload the WM without killing all your windows
* Per-window tweaks
* Bar support
