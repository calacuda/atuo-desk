import re

expr = "-(([0-9]+)*[0-9]+)"
foo = "Chrome_ChildIOT-228588  [007] ..... 17443.607545: inet_sock_set_state: family=AF_INET6 protocol=IPPROTO_TCP sport=46878 dport=443 saddr=0.0.0.0 daddr=0.0.0.0 saddrv6=2603:7000:a100:7c50:494e:156f:8eea:f335 daddrv6=2602:fd3f:3:ff02::2f oldstate=TCP_ESTABLISHED newstate=TCP_CLOSE"

m = re.search(expr, "Chrome_ChildIOT-228588  [007] ..... 17443.607545: inet_sock_set_state: family=AF_INET6 protocol=IPPROTO_TCP sport=46878 dport=443 saddr=0.0.0.0 daddr=0.0.0.0 saddrv6=2603:7000:a100:7c50:494e:156f:8eea:f335 daddrv6=2602:fd3f:3:ff02::2f oldstate=TCP_ESTABLISHED newstate=TCP_CLOSE")
print(m[1])

m = re.search(" saddr=([^\s]+) daddr=([^\s]+)", foo)
print(m[1], m[2])

m = re.findall("[s|d]port=([0-9]+)", foo)
print(m)

m = re.findall("sport=([0-9]+) dport=([0-9]+)", foo)
print(m)
