Gallium
=======

A WM written in Rust, inspired and based loosely off of Kintaro's wtftw project.
This is a toy project. Don't even think about using it.

Currently supports:
* Customizable keybindings
* Tall tiling mode and fullscreen mode
* Gaps and padding for windows
* Workspaces
* ???

Still to do:
* Better color serialization/config
* Switch to TOML(YAML?) for config over JSON
* More keybinds (sending windows across workspaces, etc.)
* Hot-reloading of WM without losing all your windows
* Probably rethink how layouts work since they kinda suck??
* More layout options (BSP?)
* Floating windows
* Per-window tweaks
* EWMH exporting

There should probably be core 5 Layouts - Tall, Wide, Grid, Focused, and Floating.
Tall tiles vertically, Wide horizontally, Grid will have a resizable number of rows and columns with an overflow square in the bottom-right, Focused will full-screen the currently selected window and hide the rest, and Floating will remap all existing windows and any new ones to floating, but switch them back when switched to another layout.
