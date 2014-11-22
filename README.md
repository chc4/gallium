A WM written in Rust, inspired and based loosely off of Kintaro's wtftw project.
This is a toy project. Don't even think about using it.

Brainstorming:
Each workspace will have a Layout, a deck of tiling windows, a deck of floating windows, and probably a deck of windows in the Master area.
Each deck has a current window selected, and a stack of windows that are drawn under it.
To make that work with M4-j/k scrolling through the list of windows, it won't actually pull the newly selected window to the top of the stack, only setting it to the current. If you explicitly switch to another window by clicking it (or mousing over it if that's enabled) it will be pulled to the top instead.

There should probably be the core 5 Layouts - Tall, Wide, Grid, Focused, and Floating.
Tall tiles vertically, Wide horizontally, Grid will have the Master area arrayed in a resizable center area with the remaining windows tiled around it, Focused will full-screen the currently selected window and hide the rest, and Floating will remap all existing windows and any new ones to floating, but switch them back when switched to another layout.

The bar will probably just be a plug-n-play bar, with the WM exporting the current WM, window selected, etc. Not sure if you can do some info bars would like without XCB though.

Config will be hotswappable, and probably the entire WM as well. The config will just be in JSON format since there is already a nice serializer for it in place.
Keybinds will be in Emac's format since it's an easy way to say them. M4-r means "The Super key+r". You can add as many modifiers as you want ("S-","Lock-","C-","M-","M2-","M3-","M4-","M5-" are the options. Shift, Lock, Control, Meta(alt), Meta2-5. All keybinds will be defined this way, and reloadable from the config file.
