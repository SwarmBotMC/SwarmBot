# Structure

## Commands

All commands sent to SwamBot are created from a websocket using JSON message. Currently, the only full command implementation is the Kotlin Minecraft mod but any language supporting JSON and WebSockets should be able to control the bots.

### Format

```json5
{
    path: "{command idenitifier}",
    // ...
}
```
where `...` represents other fields. For example `GoTo` can be represented as 
```json5
{
    path: "goto",
    location: {
        x: 123,
        y: 233,
        z: 323
    }
}
```

### Command List
Look at the [`Command enum`](https://github.com/andrewgazelka/SwarmBot/blob/4af097100206db7a7c8e651faaeec2bd43ec21e8/src/client/commands.rs#L73-L77). All
structs are deserialized with serde and are in the format one would expect given the struct.
