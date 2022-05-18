"""
bspwm.py

fucntions to do things in bspwm, bassically a wrapper for bspc


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


# from os import system
import common as com


def open_on_desktop(program, desktop) -> int:
    """
    opens a program on a desktop
    """
    prerun = ["monitor -a Desktop",
              "desktop Desktop -f"]
    postrun = [f"node -d {desktop} --follow",
               "desktop Desktop --remove"]

    for cmd in prerun:
        com.send(cmd)

    com.open_program(program)

    for cmd in postrun:
        com.send(cmd)

    return 0


def close_focused():
    """closes the curently focused window"""
    return com.send('node -c')


def move_to(destination):
    """moves the focused window to the destination desktop"""
    return com.send("node -d " + desktop)


def focus_on(destination):
    """focuse on the destination desktop"""
    return com.send("node -f " + desktop)


controls = {'open-at': open_on_desktop,
            'close-focused': close_focused,
            'move-to': move_to,
            'switch-to': focus_on}
