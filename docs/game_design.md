# Overview

Server Looter is a incremental game set in a dreary cyberpunk setting where 
players utilize servers they own to hack other machines and protect their own.
The game is primarily singleplayer, where players attempt to hack corporations,
but players may also opt-in to PvP by connecting to the "darknet".

A player hacks a server by executing "scripts", which will result in the server
attempting to disconnect the player. Scripts may be oriented towards extracting
resources, such as credits (money) or new script "algorithms".

## Servers

Servers are computers owned by the player. These computers run scripts and other
processes that are useful, and players will strive to purchase both stronger
servers (in terms of compute = faster execution) and more servers 
(in terms of capacity = more concurrent operations).

Servers exist in a server rack, where they share power and bandwidth with all
other servers. If exceeding 100% power utilization, servers will underclock
appropriately to reach 100% power utilization, meaning scripts and other 
operations running on those servers will run slower.

## Scripts + Algorithms

Scripts are collections of algorithms that are run to hack servers.

Algorithms are different units of functionality that are combined to do 
useful things. For example, a resource siphon algorithm might steal credits
from the target server, while a detection suppression algorithm might slow or
prevent the server from disconnecting the hacker.

Scripts combine these algorithms into runnable programs that are used to hack
servers. A script may consist of one or more algorithms, and these algorithms may
be run in parallel or in series. The maximum parallelization of a script dictates
the number of threads required to run it. For example, to run three algorithms
concurrently, the server running the script must have at least three threads.
A script cannot be run on a server with fewer threads than required.
