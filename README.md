# MineBot

A bot launcher for Minecraft written in Rust.

## MC Versions

- 1.12.* ✅
- 1.16 — planned

## Running

See `./minebot --help`

## Configuring

You will need two files in the current working directory. **Make sure proxies are not hella sketch**,
they are used for Mojang authentication as well as logging in. If Mojang deems your proxy sketch, the
alt account may get locked.

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
