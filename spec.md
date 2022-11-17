# Specifications and technical details
----

## sending commands to the server:
format: `command arg1 arg2 arg3`
~~if a single argument has spaces in it than they should be encoded as `\s`.
in this example: `open-here tmux\snew\s-d\s-s\sfoobar` the `open-here` command gets `tmux new -d -s foobar` as inputs. this runs the tmux command as a shell command and then exits. this slash s encoding is done to make parsing the commands easier, simpler, and more reliable.~~ (not yet implemented)

once a command is sent to the server it will reply with an exit code (see below)

## universal commands:
|command | arguments | description |
|--------|-----------|-------------|
|open-here | cmd | runs command (or launches the program) and then waits for it launch before returning. |
|poweroff | N/A | powers off the system via systemctl poweroff|
|hibernate | N/A | hibernates the system via systemctl hibernate|
|reboot | N/A | reboots the system via systemctl reboot|
|sleep OR suspend | N/A | suspends the system via systemctl suspend-then-hibernate|
|lock | N/A | locks the system via loginctl lock-session|
|logout | N/A | logs out of the current session via loginctl|
|vol-up | percent | raises system volume by percent|
|vol-down | percent | lowers system volume by percent|
|mute | N/A | mutes system audio|
|play/pause | N/A | toggles from play to pause and vice versa  |
|play-track | N/A | plays paused media|
|pause-track | N/A | pauses playing audio|
|stop-track | N/A | stops current media|
|next-track | N/A | skips to next media|
|last-track | N/A | skips to last media|
|inc-bl | percent | increases the screen backlight brightness by percent|
|dec-bl | percent | decreases the screen backlight brightness by percent|
|load-layout | layout | is a .layout file in ~/.config/desktop-automater/layouts/ dir, it contains a new line separated list of commands to be run. (under active development))

## bspwm commands:
|command | arguments | description |
|--------|-----------|-------------|
|`open-at`/`open-on` | desktop, program | opens a program on the specified desktop then waits for the program to launch before continuing. |
|`close-focused` | N/A | closes the currently focused node (window).|
|`move-to` | desktop | moves the currently focused node to the specified desktop|
|`focus-on` | desktop | switches focus to the specified desktop|
|`add-mon` | monitor | turns monitor on (does not position use `add-mon-r` (add-mon-r not yet implemented) or similar)|

## qtile commands:
|command | arguments | description |
|--------|-----------|-------------|
| `open-at`/`open-on` | `exe`, `wm_class`, `desktop` | runs the program, `exe`, with the window manager class, `wm_class`, on the desktop, `desktop`. |
| `load-layout` | `layout` | sets up the layout, `layout`. |
| `focus-on` | `workspace` | switches focus to the group `workspace`. |

## exit Code:
|code | description |
|-----|-------------|
|0    |  no errors
|1    |  command not found
|2    |  error running command, check logs
|3    |  process error-ed out while executing supporting command.
|4    |  there was an error with the main command
|5    |  error connecting to wm socket
|6    |  wm error
|7    |  too few/many arguments

---
# design note:
---
## plug-in system:

add work spaces and features