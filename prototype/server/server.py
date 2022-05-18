"""
server.py

server for controlling linux desktop enviornments/window managers.


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


import socket
import os
import plugins.bspwm as desktop
import plugins.common as com
import signal
import sys
import configparser


# PATH = "/tmp/desktop-control"  # assigned later
CONTROLS = {**desktop.controls, **com.controls}


def ensure_file(config_file):
    """makes default config if its not existent."""
    if not os.path.exists(config_file):
        with open(config_file, 'w') as f:
            f.write('[SERVER]\n')
            f.write('bspwm-so = /tmp/bspwm_0_0-socket\n')
            f.write('prog-so = /tmp/desktop-automater\n')


def ensure_dir(file_path):
    """makes file paths"""
    # for path in file_path.split('/')[:-1]:
    #     if not os.path.exists(path):
    #         os.mkdir(path)
    path = os.path.dirname(file_path)
    if not os.path.exists(path):
        ensure_dir(path)
        os.mkdir(path)


def get_config(config_file):
    """gets the config file if it exist, else it makes it"""
    config_file = os.path.expanduser(config_file)

    if not os.path.exists(config_file):
        ensure_dir(config_file)
        ensure_file(config_file)

    config = configparser.ConfigParser()
    config['SERVER'] = {
        'bspwm-socket': '/tmp/bspwm_0_1-socket',
        'program-socket': '/tmp/desktop-automater'
    }

    config.read(config_file)
    return config


def exit_gracefully(signum, frame):
    os.remove(PROG_SOCK)
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
        res = func(BSPWM_SOCK, *cmd[1:])
    except Exception as e:
        print(e)
        res = 2

    return res


def make_msg(msg) -> bytes:
    """
    takes a message and returns it in a form that can be sent over a unix socket
    """
    print("error code : ", msg)
    return bytes(str(msg), 'utf-8')


def main():
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.bind(PROG_SOCK)

        while True:
            try:
                s.listen(1)
                conn, adr = s.accept()
                msg = conn.recv(1024).decode('utf-8')
                print(f"got {msg}")
                code = make_msg(switch_boad(msg))
                conn.send(code)
                # print("error code : ", code)
            except (KeyboardInterrupt, SystemExit):
                print()
                break

    if os.path.exists(PROG_SOCK):
        os.remove(PROG_SOCK)


if __name__=="__main__":
    CONFIG = get_config("~/.config/desktop-automater/config.ini")

    PROG_SOCK = CONFIG['SERVER']['program-socket']
    BSPWM_SOCK = CONFIG['SERVER']['bspwm-socket']

    if os.path.exists(PROG_SOCK):
        os.remove(PROG_SOCK)

    CONTROLS['kill'] = exit_gracefully
    signal.signal(signal.SIGINT, exit_gracefully)
    signal.signal(signal.SIGTERM, exit_gracefully)
    main()
