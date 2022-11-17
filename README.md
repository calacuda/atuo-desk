# desktop-automater
---
Controls Linux tiling window managers and desktop environments. It uses Unix sockets to handle communication with the client program. Development is currently focused on the tiling window manager [Qtile](http://www.qtile.org/).

---

## overview:

Desktop automator designed to allow programmatic control of the desktop environments and general system stuff. It runs as a service in the background and is controlled via a client; the two communicate over a [Unix Domain Socket](https://en.wikipedia.org/wiki/Unix_domain_socket). The project was started as an excuse to experiment with Unix Sockets, but a few additions to the original scope have made it quite useful! As of now it is only compatible with [Qtile](http://www.qtile.org/) and [BSPWM](https://github.com/baskerville/bspwm) as those are the environment I use. However support for more tiling window managers, and (maybe) floating window managers is planned for the future.

## advantages:

- configuring of [layouts](#layouts) that can be set up with ease. 
- consistent API and endpoint makes controlling the system and some programs easy.
- it's programmatic so you can write your own scripts to control the WM/DE and some common programs.

## layouts:

layouts allow for opening apps on specific workspaces, and optionally clearing that workspace. They also allow for running of arbitrary commands and will soon have the ability to tear them selfs down.

## documentation:

see [spec.md](spec.md)

## dependencies:

- systemd
- loginctl 
- alsa
- playerctl 
- xbacklight
- xrandr
- gtk-launch
- xdotool

## planned features/ideas for the future:
(for the future)

- [x] 1. write a [Mycroft](https://mycroft-ai.gitbook.io/docs/) skill to add a voice control feature
- [ ] 2. support for finding bspwm nodes by name. (so one could say, "go back to alacritty/firefox" in the mycroft skill.)
- [ ] 3. write a rofi script to search through and select layouts.

## TODOs:
(things planned for the immediate/foreseeable future)

- [ ] change between full screen, tiled, floating, and pseudo_tiled.
- [x] add simple xrandr/autorandr controls
- [x] restructure directories to be more rusty.
- [ ] add pass through for querying BSPWM.
- [ ] add better documentation. (ongoing)
- [x] make layout configured with yaml
- [ ] make layout function idempotent (add an option to only open the program if not already open))
- [ ] give layouts the ability to tear them selves down  
- [ ] write an ensure_file and make default configs file
- [x] when launching a program make it use `gtk-launch` to launch desktop files.
- [x] make setting of layouts multi-threaded.
- [ ] write a rofi script to select between layouts.
- [x] add Qtile support.

## development history and schedule:

- [x] 1. make prototype/proof of concept.
- [x] 2. rewrite in Rust/Golang. (this was done in Rust)
- [x] 3. add misc features.
- [x] 4. write systemd service
- [x] 5. add mycroft support ([repo](https://github.com/calacuda/mycroft-linux-control-skill))
- [ ] 6. add python module to be imported from qtile conf. <= we are working on this. (everything below this is subject to change)
- [ ] 7. add finishing touches to bspwm support
