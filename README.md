# MineBot

A bot launcher for Minecraft written in Rust. 

## What makes this unique?

- **Performant**. I am able to run hundreds of bots off of my 2015 laptop with under 10% CPU. This is because MineBot does not depend on the default Minecraft client. Instead, it completely re-implements physics and the Minecraft protocol in Rust which allows for increadibly fast speeds.
- **Easy**. It is very easy to launch as many bots as you want. Simply do `./minebot -c {number} {server ip}`,

## Features
- Incremental path navigation ✅ — `#goto`
- Mining ✅ `#mine` — mines in 7×y×7 regions, where y is the highest block in the chunk
- Parkour ✅ the best bot for parkouring at bedrock that I know of.
- Bucket falling ✅

## MC Versions
If you want to support a version you will need to implement the `Minecraft` trait for that version.
- 1.12.* ✅
- 1.16 — planned

## Running

See `./minebot --help`

## Configuring

You will need two files in the current working directory. **Make sure proxies are not hella sketch**,
they are used for Mojang authentication as well as logging in. If Mojang deems your proxy sketch, the
alt account may get locked. Proxies are recommended as Mojang rate limits auth requests.

- `proxies.csv` a CSV (separated by `:`) of proxies `ip:port:user:pass`
- `users.csv` a CSV (separated by `:`) of users `email:pass`

both CSVs have no header.


# Structure 

As of `d4b6d27444347a2bb54f82d212b1ad5a70126edf` the structure is as follows

|Type|Path|
|-------|----------|
A* progressions| `moves.rs`|
A* | `pathfind/incremental/mod.rs`
Physics | `physics/mod.rs`
Path follower | `follow/mod.rs`
Commands |`bot.rs`
1.12 Protocol |`v340/mod.rs`
Runner |`runner.rs`
