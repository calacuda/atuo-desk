# Spec Sheet

----

## sending commands to the server:
format: `command arg1 arg2 arg3`
if a single argument has spaces in it than they should be encoded as `\s`.
in this example: `open-here tmux\snew\s-d\s-s\sfoobar` the `open-here` command gets `tmux new -d -s foobar` as inputs. this runs the tmux command as a shell command and then exits. this slash s encoding is done to make parsing the commands easier, simpler, and more reliable.

once a command is sent to the server it will replay with an exit code (see below)

## universal commands:
|command | arguments | description |
|--------|-----------|-------------|
|open-here | cmd, delay(optional) | runs command (or launches the program) and then waits delay (default delay is 0.2)

## bspwm commands:
|command | arguments | description |
|--------|-----------|-------------|
|open-at | program, desktop, delay(optional) | runs a program in a tmp desktop then moves it to the desktop arg. delay can be increased to ensure that the program gets to the desired desktop.  
|close-focused | N/A | closes the currently focused node (window).
|move-to | desktop | moves the currently focused node to the specified desktop
|switch-to | desktop | switches focus to the specified desktop

## exit Code:
|code | description |
|-----|-------------|
|0    |  no errors
|1    |  command not found
|2    |  python error running command, check logs
|3    |  process error-ed out while executing supporting command.
|4    |  there was an error with the main command
