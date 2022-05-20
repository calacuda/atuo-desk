# desktop-controller
Controls Linux desktop environments and tiling window managers. It uses Unix sockets to handle communication with the client program. Development is currently focused on the [BSPWM](https://github.com/baskerville/bspwm) tiling window manager.

---

## overview:

Desktop automator designed to allow programmatic control of the desktop environments and general system stuff. It runs as a service in the background and is controlled via a client; the two communicate over a [Unix Domain Socket](https://en.wikipedia.org/wiki/Unix_domain_socket) (UDS, IPC). The project was started as an excuse to experiment with Unix Sockets, but with a few additions could be quite useful! As of now it is only compatible with [BSPWM](https://github.com/baskerville/bspwm) as that is the primary environment I currently use. However support for desktop environments (DE), more tiling window managers, and floating window managers is planned for the future.

## advantages:

- consistent API and endpoint makes controlling the system and some programs easy.
- it's programmatic so you can write your own scripts to control the WM/DE and some common programs.

## dependencies:
- systemd
- loginctl

## planned features:
- [ ] 1. write a [Mycroft](https://mycroft-ai.gitbook.io/docs/) skill to add a voice control feature
- [ ] 2. add basic spotify support
- [ ] 3. add VERY basic web browser control. (prob using the gecko/chromium drives. this would make it easier to support firefox/chromium and most derivatives with the same code base.)
- [ ] 4. support for finding bspwm nodes by name. (so one could say, "go back to alacritty/firefox" in mycroft.)

## TODOs:
(things planned for the immediate/foreseeable future)

- [x] 1. ~~add config file~~
- [ ] 2. write system service
- [x] 3. rewrite is rust (and or GO)
- [ ] 4. add support for KDE
- [ ] 5. change between full screen, tiled, floating, and pseudo_tiled.
- [ ] 6. power control (sleep, hibernate, poweroff, lock, etc)
- [ ] 7. system restart service

## development history and schedule:

- [x] 1. make prototype/proof of concept.
- [x] 2. rewrite in Rust/Golang. (this was done in Rust)
- [ ] 3. write systemd service <= we are here. (everything below this is subject to change)
- [ ] 4. finding BSPWM node by name
- [ ] 5. add support for KDE.
