####################################
# Auto-Desk API stuff helper hooks #
####################################
"""
auto_desk_api.py

an api helper for the desktop-automator project (https://github.com/calacuda/desktop-automater)
this needs to be imported in you're qtile/config.py file. for desktop automator to be fully compatible.


By: Calacuda | MIT License | Python Version: 3.9, 3.10
"""



from libqtile.command.client import InteractiveCommandClient
from libqtile.log_utils import logger
from libqtile import hook
# import os
from socket import socket, AF_UNIX, SOCK_STREAM
# import asyncio



QTILE_CLIENT = InteractiveCommandClient()
NEW_CLIENT_PIDs = set()
PATH = "/tmp/desktop-automater"


# @hook.subscribe.client_managed
# async def open_on_bak(c):
#     move_window(c)


def _open_on(client):
    """used to move windows when they open""" 
    pid = client.get_pid()

    # this function gets called twice per window opening.
    # bellow stops this function from moving windows that have already been moved.
    if pid not in NEW_CLIENT_PIDs:
        NEW_CLIENT_PIDs.add(pid)
        move_window(client)
    else:
        NEW_CLIENT_PIDs.remove(pid)


@hook.subscribe.client_managed
async def open_on_backup(client):
    """
    used to move windows when the program sets its WM_CLASS after its managed (ie, after its registered)
    this was made bc spotify doesn't like playing nice with linux.

    this a back up for open_on().
    """
    _open_on(client)


@hook.subscribe.client_new
async def open_on(client):
    """moves windows when they register"""
    _open_on(client)


@hook.subscribe.group_window_add
async def clear_group(group, window):
    clearing = should_clear(group.name)
    if clearing:
        logger.debug(f"clearing group {group.name}")
        pid = window.get_pid()
        for w in group.windows:
            if w.get_pid() != pid:
                w.togroup("hidden")


def get_location(wm_class):
    message = f"auto-move {wm_class[0]} {wm_class[1]}"
    return send_auto_desk(message)


def should_clear(group):
    message = f"should-clear {group}"
    res = send_auto_desk(message)
    logger.debug(f"should-clear res: '{res}'")
    return res == "true"


def send_auto_desk(message):
    """sends data to auto-desk and returns the response"""
    location = None

    with socket(AF_UNIX, SOCK_STREAM) as s:
        s.settimeout(10)
        try:
            s.connect(PATH)
        except FileNotFoundError:
            pass
        except TimeoutError:
            pass
        else:
            s.send(bytes(message, "utf-8"))
            s.shutdown(1)  # tells the server im done sending data and it can reply now.
            res = s.recv(1024)
            ec = res[0]
            if len(res) >= 3:
                location = res[2:].decode('utf-8')
            if ec:
                logger.error(f"got error code from auto-desk on message '{message}'.")

    return location


def clear_desktop(group):
    """clears all desktop in self.clears"""
    # for group in tmp_clears:
    if group:
        # windows = [w["id"]
                # for w in QTILE_CLIENT.windows() if w["group"] == group]
        windows = QTILE_CLIENT.group[group].windows()
        logger.info(f"about to clear windows from group '{group}'")
        for wid in windows:
            QTILE_CLIENT.window[wid].togroup("hidden")
        logger.info(f"cleared group '{group}'")
    else:
        logger.info(f"not clearing group '{group}'")            


def move_window(c):
    wm_class = c.get_wm_class()
    location = get_location(wm_class)
    logger.debug(f"moving to location, '{location}'")
    if location:
        c.togroup(location)
