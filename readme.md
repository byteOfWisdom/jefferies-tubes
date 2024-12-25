jeffries-tubes:
---

this is a set of tools for connecting unix pipes in n to m configurations.
The goal is to be high performance enough so that the performance advantage of using plain anonymus pipes remains small.

depencencies:
---
listen script depends on socat, but the script can be substituted by a small program if that is preferred (but socat is open source and easy and performs very well).
Build depencies for pipe_mux are just cargo or, if that must be avoided the bus library and a recent rust compiler.

installation:
---
Build pipe_mux using cargo (or your preferred way of building a rust project with dependencies. but i'd use carg ;P)
and add the produced binary and the listen script to your path.

usage:
---
producing processes pipe their stdout into "pipe_mux [tag]", where tag is a name, which should be unique but otherwise can be arbitrary.

consuming processes can subsrice to tags by using "listen [tag]". this i non blocking with regargs to the producing process (i.e. not consuming stdin fast enough or at all will not influence anything apart from the consuming process, which might not get sent all data)
