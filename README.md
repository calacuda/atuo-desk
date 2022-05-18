# desktop-controller
A program that controls Linux desktop environments and tiling window managers. It uses Unix sockets to handle communication with the client program. Development is currently focused on the BSPWM tiling window manager.

---

## TODOs

- [x] 1. ~~add config file~~
- [x] 2. ~~clearly define API spec~~
- [ ] 3. write system service
- [ ] 4. add plug-in loading system
- [ ] 5. rewrite is rust (and or GO)

## plug-in system:

it will need some standard for passing data to it. 

options for data passing: 

- command line arguments for executables. (easiest and allows for abandoning python)
- if using python, we can define a consistent entry point and wrap other languages as python modules. (python reliant)  

Ideas:

- executable scripts/binaries that get sourced at run time and stored in a hash map.
- files that contain list of the commands to be sent over the socket.
