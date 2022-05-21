# Specifications and technical details
----

## sending commands to the server:
format: `command arg1 arg2 arg3`
if a single argument has spaces in it than they should be encoded as `\s`.
in this example: `open-here tmux\snew\s-d\s-s\sfoobar` the `open-here` command gets `tmux new -d -s foobar` as inputs. this runs the tmux command as a shell command and then exits. this slash s encoding is done to make parsing the commands easier, simpler, and more reliable.

once a command is sent to the server it will replay with an exit code (see below)

## universal commands:
|command | arguments | description |
|--------|-----------|-------------|
|open-here | cmd | runs command (or launches the program) and then waits delay (default delay is 0.2)
|poweroff | N/A | powers off the system via systemctl poweroff
|hibernate | N/A | hibernates the system via systemctl hibernate
|reboot | N/A | reboots the system via systemctl reboot
|sleep OR suspend | N/A | suspends the system via systemctl suspend-then-hibernate
|lock | N/A | locks the system via loginctl lock-session
|logout | N/A | logs out of the current session via loginctl
|vol-up | percent | raises system volume by percent
|vol-down | percent | lowers system volume by percent
|mute | N/A | mutes system audio
|play/pause | N/A | toggles from play to pause and vice versa  
|play-track | N/A | plays paused media
|pause-track | N/A | pauses playing audio
|stop-track | N/A | stops current media
|next-track | N/A | skips to next media
|last-track | N/A | skips to last media
|inc-bl | percent | increases the screen backlight brightness by percent
|dec-bl | percent | decreases the screen backlight brightness by percent

## bspwm commands:
|command | arguments | description |
|--------|-----------|-------------|
|open-at | desktop, program | runs a program in a tmp desktop then moves it to the desktop arg. delay can be increased to ensure that the program gets to the desired desktop.  
|close-focused | N/A | closes the currently focused node (window).
|move-to | desktop | moves the currently focused node to the specified desktop
|focus-on | desktop | switches focus to the specified desktop
|add-mon | monitor | turns monitor on (does not position use `add-mon-r` or similar)

## exit Code:
|code | description |
|-----|-------------|
|0    |  no errors
|1    |  command not found
|2    |  error running command, check logs
|3    |  process error-ed out while executing supporting command.
|4    |  there was an error with the main command
|5    |  error connecting to BSPWM socket
|6    |  BSPWM error
|7    |  too few arguments

---
# design note:
---
## plug-in system:

it will need some standard for passing data to it.

options for data passing:

- command line arguments for executables (either text or bin). (easiest and allows for abandoning python)
- ~~if using python, we can define a consistent entry point and wrap other languages as python modules. (python reliant)~~

Ideas:

- executable scripts/binaries that get sourced at run time and stored in a hash map.
- files that contain list of the commands to be sent over the socket.
