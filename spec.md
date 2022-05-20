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
|open-here | delay(optional), cmd | runs command (or launches the program) and then waits delay (default delay is 0.2)

## bspwm commands:
|command | arguments | description |
|--------|-----------|-------------|
|open-at | desktop, delay(optional), program | runs a program in a tmp desktop then moves it to the desktop arg. delay can be increased to ensure that the program gets to the desired desktop.  
|close-focused | N/A | closes the currently focused node (window).
|move-to | desktop | moves the currently focused node to the specified desktop
|focus-on | desktop | switches focus to the specified desktop

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
