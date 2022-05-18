"""
server.py

server for controlling linux desktop enviornments/window managers.


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


import socket
import os
import plugins.bspwm as desktop
import common as com
import signal
import sys


PATH = "/tmp/desktop-control"
CONTROLS = {**desktop.controls, **com.controls}


def exit_gracefully(signum, frame):
    os.remove(PATH)
    sys.exit(0)


def switch_boad(cmd: str) -> int:
    """
    takes the command form the client and returns 0 if command ran sucefully.
    otherwise it returns an error code.
    """
    cmd = cmd.split(" ")
    cmd = [token.strip().replace('\s', ' ') for token in cmd]
    print(cmd)
    func = CONTROLS.get(cmd[0])
    if not func:
        return 1

    try:
        return func(*cmd[1:])
    except Exception as e:
        print(e)
        return 2

    # return 0


def make_msg(msg) -> bytes:
    """
    takes a message and returns it in a form that can be sent over a unix socket
    """
    print("error code : ", msg)
    return bytes(str(msg), 'utf-8')


def main():
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.bind(PATH)

        while True:
            try:
                s.listen(1)
                conn, adr = s.accept()
                msg = conn.recv(1024).decode('utf-8')
                print(f"got {msg}")
                code = make_msg(switch_boad(msg))
                conn.send(code)
                # print("error code : ", code)
            except KeyboardInterrupt:
                print()
                break

    if os.path.exists(PATH):
        os.remove(PATH)


if __name__=="__main__":
    if os.path.exists(PATH):
        os.remove(PATH)
    CONTROLS['kill'] = exit_gracefully
    signal.signal(signal.SIGINT, exit_gracefully)
    signal.signal(signal.SIGTERM, exit_gracefully)
    main()
