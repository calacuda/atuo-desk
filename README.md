# desktop-automater
Controls Linux desktop environments and tiling window managers. It uses Unix sockets to handle communication with the client program. Development is currently focused on the [BSPWM](https://github.com/baskerville/bspwm) tiling window manager.

---

## overview:

Desktop automator designed to allow programmatic control of the desktop environments and general system stuff. It runs as a service in the background and is controlled via a client; the two communicate over a [Unix Domain Socket](https://en.wikipedia.org/wiki/Unix_domain_socket) (UDS, IPC). The project was started as an excuse to experiment with Unix Sockets, but with a few additions could be quite useful! As of now it is only compatible with [BSPWM](https://github.com/baskerville/bspwm) as that is the primary environment I currently use. However support for desktop environments (DE), more tiling window managers, and floating window managers is planned for the future.

## advantages:

- consistent API and endpoint makes controlling the system and some programs easy.
- it's programmatic so you can write your own scripts to control the WM/DE and some common programs.

## documentation:

see [spec.md](spec.md)

## dependencies:
- systemd
- loginctl
- alsa (plans to remove in future)
- playerctl
- xbacklight (plans to remove in future)
- xrandr
- gtk-launch
- coproc

## planned features/ideas for the future:
(for the future)

- [x] 1. write a [Mycroft](https://mycroft-ai.gitbook.io/docs/) skill to add a voice control feature
- [ ] 2. support for finding bspwm nodes by name. (so one could say, "go back to alacritty/firefox" in the mycroft skill.)
- [ ] 3. support for KDE.

## TODOs:
(things planned for the immediate/foreseeable future)

- [ ] add support for KDE
- [ ] change between full screen, tiled, floating, and pseudo_tiled.
- [x] add simple xrandr/autorandr controls
- [ ] add pass through for querying BSPWM.
- [ ] add better documentation. (ongoing)
- [x] make layout configured with yaml
- [ ] make layout function idempotent
- [ ] write an ensure_file and make default configs file
- [ ] when launching a program make it use `gtk-launch` (maybe also `coproc`?) to launch desktop files.

## development history and schedule:

- [x] 1. make prototype/proof of concept.
- [x] 2. rewrite in Rust/Golang. (this was done in Rust)
- [x] 3. add misc features.
- [x] 4. write systemd service
- [x] 5. add mycroft support
- [ ] 6. finish bspwm support <= we are working on this. (everything below this is subject to change)
- [ ] 7. add support for KDE
