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
s.send(b"add-hook port-status-change notify-send \"~$local_adr~ => ~$remote_adr~\" \"exe :  ~$executable~\" ")
# s.send(b"add-hook wifi-network-change notify-send \"from: $from => to: $to\" ")
# s.send(b"add-hook test_file_exists /tmp/file-notif")
# s.send(b"load-layout TEST")
# s.send(b"load-layout coding")
# s.send(b"open-on brave brave-browser 5")
# s.send(b"lock")
s.shutdown(1)  # tells the server im done sending data and it can reply now.
# s.setblocking(True)
res = s.recv(1024).decode('utf-8') 
print(res)
# print(dir(s))
# print(f"got: {s.recv(1024)}")
s.close()
