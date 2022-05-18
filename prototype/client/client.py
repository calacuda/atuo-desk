"""
client.py

client for controlling linux desktop enviornments/window managers.


By: Calacuda | MIT Licence | Epoch: May 18, 2022
"""


import socket


PATH = "/tmp/desktop-automater"
# PATH = "/tmp/bspwm_0_0-socket"

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(PATH)
# s.connect()
s.send(b"open-at pokemmo-launcher 9")
# print(f"got: {s.recv(1024)}")
print(f"{s.recv(1024).decode('utf-8')}")
