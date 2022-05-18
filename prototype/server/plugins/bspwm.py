"""
bspwm.py

fucntions to do things in bspwm, bassically a wrapper for bspc


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


# from os import system
import plugins.common as com


def open_on_desktop(spath, program, desktop, delay=0) -> int:
    """
    opens a program on a desktop
    """
    prerun = ["monitor -a Desktop",
              "desktop Desktop -f"]
    postrun = [f"node -d {desktop} --follow",
               "desktop Desktop --remove"]

    for cmd in prerun:
        com.send(spath, cmd)

    com.open_program(spath, program, int(delay))

    for cmd in postrun:
        com.send(spath, cmd)

    return 0


def close_focused(spath):
    """closes the curently focused window"""
    return com.send(spath, 'node -c')


def move_to(spath, destination):
    """moves the focused window to the destination desktop"""
    return com.send(spath, "node -d " + desktop)


def focus_on(spath, destination):
    """focuse on the destination desktop"""
    return com.send(spath, "node -f " + desktop)


controls = {'open-at': open_on_desktop,
            'close-focused': close_focused,
            'move-to': move_to,
            'switch-to': focus_on}
