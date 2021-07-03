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


# Planned 2b2t Procedures

## Mining a chunk

1. Go to the center (x,z) of the chunk (does not matter biased towards which side)
2. Pillar to the highest block we want to mine
3. Partition into layers
4. For each layer find the furthest block away from the pillar
- (a) If the furthest block can be mined without moving mine that and all blocks without moving
- (b) Else go to as close to the pillar as possible to mine the furthest block. Then mine all of the blocks within reach that are further than the current location standing that are in reach. After that go to (a)
5. Go to pillar and mine one block down

## Partitioning Chunks

**Might need to add a global task for this**.

1. Define rectangular regions from chunks `(x1, z1)` to `(x2,z2)`.
2. Create a region from a union of the rectangular regions.
3. Define an ordering policy. i.e., furthest from origin first or closest to origin first
4. Assign a **number** to each chunk in region (+1 for each manhatten distance from origin away). 
5. Suppose we have ordering closest to furthest. The first bot will get 0, the second, third, forth, fifth bot will get 1, sixth will get 2, .. etc.

Example:
```text
5432345
4321234
3210123
4321234
5432345
```

### Progressing
Once a bot has finished mining a chunk it will need to progress. To the next one.
This will be done by choosing the smallest **number** (assuming closest first else would be largest) that is adjacent to the bot. This will prevent the bot from traveling a large distance to mine the small/larg**est** number.
