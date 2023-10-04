# desktop-automater

---

Controls Linux tiling window managers and desktop environments. It uses Unix sockets to handle communication with the client program. Development is currently focused on the tiling window manager [LeftWM](https://leftwm.org/).

---

## overview

Desktop automator designed to allow programmatic control of the desktop environments and general system stuff. It runs as a service in the background and is controlled via a client; the two communicate over a [Unix Domain Socket](https://en.wikipedia.org/wiki/Unix_domain_socket). The project was started as an excuse to experiment with Unix Sockets, but a few additions to the original spec have made it quite useful! As of now it is only compatible with [LeftWM](https://leftwm.org/), [Qtile](http://www.qtile.org/), and [BSPWM](https://github.com/baskerville/bspwm) as those are the environment I use. However support for more tiling window managers, and (maybe) floating window managers is planned for the future.

## advantages

- configuring of [layouts](#layouts) that can be done with ease.
- consistent API and endpoint makes controlling the system easy.
- simple programatic API so you can write your own scripts to control the WM/DE and system.
- consistant across different WM and DE allowinf one script to work across mutiple enviornemnts.
- checks for system [events](#event hooks) allowing you to hook your own commands to run when the system state changes.

## layouts

layouts allow for opening apps on specific workspaces, and optionally clearing that workspace. They also allow for running of arbitrary commands and will soon have the ability to tear them selfs down.

## event hooks

Event hooks allow you to have you own programs or commands run on system events or system state chagnes.
these events include:

- wifi network changes
- bluetooth device connection/disconnection
- USB device connection/disconnection
- network port status change
- network connection/disconnection

## Suported Window Managers

- [BSPWM](https://github.com/baskerville/bspwm) (support is old but _should_ still work)
- [Qtile](http://www.qtile.org/) (support is old but _should_ still work)
- [LeftWM](https://leftwm.org/) (tested and working as intended)

## documentation

see [spec.md](spec.md)

## dependencies

- systemd
- loginctl
- alsa
- playerctl
- xbacklight
- xrandr
- gtk-launch
- xdotool

## currnet state

I would describe the currnet state as development but good enough for bassic usage.

## port status

There is a temporary solution for detecting changes in port status for event hooks. However, **I DO NOT LIKE THIS SOLUTION AND IT IS TEMPORARY** as it is a combersum, cobled together from popsicle sticks and chewing gum, and just over all not a great solution. I'm using it because is is FAR more resource efficient compared to the naive solution -- that is to say, checking port status in an infinite loop and comparing results. It relies on [ftrace](https://www.kernel.org/doc/html/v4.18/trace/ftrace.html) (a linux kernel module to track/trace kernel and system function calls). I intend to change this in the future but for now, it works well enough for testing.

## planned features/ideas for the future

(ideas the distant future/things I'm considering adding)

- [x] 1. write a [Mycroft](https://mycroft-ai.gitbook.io/docs/) skill to add voice control
- [x] 2. write a rofi script to search through and select layouts.
- [ ] 3. add a "procedure", feature. it will be a list of command to run and will be able to be acivated from a layout files or by its self.

## TODOs

(things planned for the immediate/foreseeable future)

- [ ] change between full screen, tiled, floating, and pseudo_tiled.
- [x] add simple xrandr/autorandr controls.
- [x] restructure directories to be more rusty.
- [ ] ~~add pass through for querying BSPWM.~~
- [ ] add better documentation. (ongoing).
- [x] make layout configured with yaml.
- [ ] make layout function idempotent (add an option to only open the program if not already open)).
- [ ] give layouts the ability to tear them selves down.
- [ ] write an install.sh file as well as default config files.
- [x] when launching a program make it use `gtk-launch` to launch desktop files.
- [x] make setting of layouts multi-threaded.
- [x] write a rofi script to select between layouts.
- [x] add Qtile support.
- [x] add leftwm support.
- [ ] send logs to client and let the client print them as well.
- [ ] add a `commands` list to the config file. this will be a list of commands to run when the layout is loaded. these commadns should be headless shell commands (commands that do not launch a gui of any sort).

## development history and schedule

- [x] 1. make prototype/proof of concept.
- [x] 2. rewrite in Rust/Golang. (this was done in Rust)
- [x] 3. add misc features.
- [x] 4. write systemd service.
- [x] 5. add mycroft support. ([repo](https://github.com/calacuda/mycroft-linux-control-skill))
- [x] 6. add python module to be imported from qtile conf.
- [x] 7. add leftwm suport.
- [x] 8. write rofi layout script. ([repo]())
- [ ] 9. finish port sensor. <= we are working on this. (everything below this is subject to change)
- [ ] 10. add finishing touches to bspwm support.
