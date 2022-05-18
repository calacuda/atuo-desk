"""
common.py

a library of common fucnitons needed by all plugins


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


from subprocess import Popen, PIPE
from time import sleep
import socket


# SPATH = "/tmp/bspwm_0_0-socket"


def to_api(cmd):
    """replaces spaces with null chars and trurns it to bytes"""
    null = bytes(chr(0), 'utf-8')
    return b''.join([bytes(tok, 'utf-8') + null for tok in cmd.split(' ')])


def send(spath, payload, error_code=4):
    """send payload over the unix socket"""
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(spath)
        s.send(to_api(payload))
        res = s.recv(1024)
        # print(str(res))
        if res.startswith(bytes(chr(7), 'ascii')):
            return error_code
    return 0


def open_program(spath, program, delay=0):
    """opens program"""
    
    Popen(program, stdout=PIPE, stderr=PIPE)
    sleep(.2 + int(delay))
    return 0


controls = {'open-here': open_program}
