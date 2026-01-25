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

# Tutorial

1. Landing screen shows a login page that autopopulates, presumably done by the
player. A fictional server management UI loads, showing the players server 
collection.
2. The player gets an in-game message from the server management UI that states
"Last login: 782 days ago. Show tutorial?", with a Yes/No option. This document
assumes the player clicks "Yes".
3. The menu sidebar is populated with a Servers section, showing a single server
titled "fe80:0070::". The player is prompted to click this server, which 
populates the main page with its data.
4. The in-game tutorial notes that the server isn't exactly industrial grade,
but is enough to start siphoning some credits. The in-game tutorial highlights
the player's credits, of which they have a meager 73. The Develop section is
added to the menu sidebar, with a "Scripts" subsection, and the player is 
prompted to click it.
5. The player is walked through the script construction process, slotting a
common-quality credit siphon algorithm into the script and finalizing it.
6. The Market tab is now available in the menu sidebar. The player is prompted
to click this, which replaces the main page with the Market.
7. The player is shown a variety of purchasables and is prompted to purchase the 
"Unsecured Corpo Server Data Dump", which unlocks the Black Hat category,
and specifically the "Exploit" action. The player is prompted to click the
"Exploit" action, which fills the main panel.
8. The player is shown 3 servers: a CorpA server, CorpB server, and CorpC server.
Each offers different rewards and encourages different scripts to maximize 
rewards. Players are prompted to click the CorpA server, which is noted to be 
best for siphoning credits (a bank or something), then click Exploit/Start/etc.
9. The player will always succeed on their first run, demonstrating what a 
successful run looks like. This will reward an algorithm that helps with CorpB.
The tutorial then prompts the player to run against CorpB, which they will always
fail to demonstrate failure and it's consequences. The tutorial will then prompt
the player to create a new script with their new algorithm to beat CorpB.