"""
client.py

client for controlling linux desktop enviornments/window managers.


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


import socket
import time


PATH = "/tmp/desktop-automater"
# PATH = "/tmp/bspwm_0_0-socket"

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(PATH)
# s.connect()
s.send(b"vol-up 5")
# s.send(b"lock")
s.shutdown(1)  # tells the server im done sending data and it can reply now.
# s.setblocking(True)
print(f"{s.recv(1024).decode('utf-8')}")
# print(dir(s))
# print(f"got: {s.recv(1024)}")
s.close()
