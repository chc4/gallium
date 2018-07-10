Gallium
=======

```
For a second Kassad thought it was another person
Wearing the chromium forcefields he and Moneta were
draped in─but only for a second. There was nothing human
about this particular quicksilver-over-chrome construct.
Kassad dreamily noted the four arms, retractable
fingerblades, the profusion of thornspikes on throat,
forehead, wrists, knees, and body, but not once did his gaze
leave the two thousand-faceted eyes which burned with a red
ﬂame that paled sunlight and dimmed the day to blood
shadows.
                                          - Hyperion Cantos
```


A WM written in Rust, inspired and based loosely off of Kintaro's wtftw project.
This is a toy project.

(I've actually been using it as my WM in an Arch VM for a few years now. It's workable, but feature-lite, and probably has bugs that I just never run into since my setup doesn't change. Also it crashes from some dialog boxes since it doesn't do ewmh hints - you'll want `export GPG_AGENT_INFO=""` at the least so it prompts from the console instead of popup.)

Currently supports:
* Customizable keybindings
* Tall tiling mode and fullscreen mode
* Gaps and padding for windows
* Workspaces
* ???

Still to do:
* More keybinds (sending windows across workspaces, etc.)
* Hot-reloading of WM without losing all your windows
* More layout options (BSP?)
* Per-window tweaks
* EWMH exporting

There should probably be core 5 Layouts - Tall, Wide, Grid, Focused, and Floating.
Tall tiles vertically, Wide horizontally, Grid will have a resizable number of rows and columns with an overflow square in the bottom-right, Focused will full-screen the currently selected window and hide the rest, and Floating will remap all existing windows and any new ones to floating, but switch them back when switched to another layout.
