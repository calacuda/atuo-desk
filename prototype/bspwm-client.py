"""
client.py

a test client for testing bspwm.


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


import socket


# PATH = "/tmp/desktop-control"
PATH = "/tmp/bspwm_0_0-socket"

# s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
# s.connect(PATH)
# # s.connect()
# s.send(b"open-at kitty 9")
# # print(f"got: {s.recv(1024)}")
# print(f"{s.recv(1024).decode('utf-8')}")


def to_api(cmd):
    """replaces spaces with null chars and trurns it to bytes"""
    null = bytes(chr(0), 'utf-8')
    return b''.join([bytes(tok, 'utf-8') + null for tok in cmd.split(' ')])


def prepare(cmd):
    return cmd.strip('bspc ')


def main():
    raw_cmd = ' '
    while raw_cmd:
        raw_cmd = input("~~> ")
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
            s.connect(PATH)
            s.send(to_api(prepare(raw_cmd)))
            res = s.recv(1024)
            print(res)



if __name__=="__main__":
    main()
