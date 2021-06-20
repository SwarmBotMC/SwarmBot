
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use std::lazy::{SyncLazy};
use std::ops::{Add, AddAssign, Sub};

use packets::*;
use packets::read::{ByteReadable, ByteReader};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::types::Origin::{Abs, Rel};

#[derive(Clone)]
pub struct PacketData {
    pub id: u32,
    pub reader: ByteReader,
}

impl PacketData {
    #[inline]
    pub fn read<T: ByteReadable>(&mut self) -> T {
        self.reader.read()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatSection {
    pub color: Option<String>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chat {
    pub extra: Option<Vec<ChatSection>>,
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct Command<'a> {
    pub player: &'a str,
    pub command: &'a str,
    pub args: Vec<&'a str>,
}


#[derive(Debug)]
pub struct PlayerMessage<'a> {
    pub player: &'a str,
    pub message: &'a str,
}

impl<'a> PlayerMessage<'a> {
    pub fn into_cmd(self) -> Option<Command<'a>> {
        static RE: SyncLazy<Regex> = SyncLazy::new(|| {
            Regex::new(r"#(\S+) (.*)").unwrap()
        });
        let capture = RE.captures(self.message)?;

        let command = capture.get(1)?.as_str();
        let args = capture.get(2)?.as_str();


        let args: Vec<_> = if args.is_empty() {
            Vec::new()
        } else {
            args.split(' ').collect()
        };

        Some(Command {
            player: self.player,
            command,
            args,
        })
    }
}

impl Chat {
    pub fn player_message(&self) -> Option<PlayerMessage> {
        static RE: SyncLazy<Regex> = SyncLazy::new(|| {
            Regex::new(r"^<([A-Za-z_]+)> (.*)").unwrap()
        });

        let text = &self.extra.as_ref()?.first()?.text;

        let captures: Captures = RE.captures(text)?;

        let player = captures.get(1)?.as_str();
        let message = captures.get(2)?.as_str();

        Some(PlayerMessage {
            player,
            message,
        })
    }
}

impl ByteReadable for Chat {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let string: String = byte_reader.read();
        let json: Chat = serde_json::from_str(&string).unwrap();
        json
    }
}

#[derive(Writable, Readable, Debug, Copy, Clone, Default)]
pub struct Location {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Sub<Location> for Location {
    type Output = Displacement;

    fn sub(self, rhs: Location) -> Self::Output {
        Displacement {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:.2}, {:.2}, {:.2}]", self.x, self.y, self.z))
    }
}

#[derive(Writable, Readable, Debug, Copy, Clone)]
pub struct Displacement {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl AddAssign<Location> for Location {
    fn add_assign(&mut self, rhs: Location) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add<Location> for Location {
    type Output = Location;

    fn add(self, rhs: Location) -> Self::Output {
        Location {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// impl Sub<Location> for Location {
//     type Output = Location;
//
//     fn sub(self, rhs: Location) -> Self::Output {
//         Location {
//             x: self.x - rhs.x,
//             y: self.y - rhs.y,
//             z: self.z - rhs.z,
//         }
//     }
// }

impl From<Location> for LocationOrigin {
    fn from(loc: Location) -> Self {
        LocationOrigin {
            x: Abs(loc.x),
            y: Abs(loc.y),
            z: Abs(loc.z),
        }
    }
}

impl Location {

    pub fn dist2(&self, loc: Location) -> f64 {
        let dx = loc.x - self.x;
        let dy = loc.y - self.y;
        let dz = loc.z - self.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn apply_change(&mut self, loc: LocationOrigin) {
        loc.x.apply(&mut self.x);
        loc.y.apply(&mut self.y);
        loc.z.apply(&mut self.z);
    }
}

#[derive(Readable, Writable, Debug)]
pub struct ShortLoc {
    dx: i16,
    dy: i16,
    dz: i16,
}

impl From<ShortLoc> for LocationOrigin {
    fn from(loc: ShortLoc) -> Self {
        LocationOrigin {
            x: Rel(loc.dx as f64 / (128.0 * 32.0)),
            y: Rel(loc.dy as f64 / (128.0 * 32.0)),
            z: Rel(loc.dz as f64 / (128.0 * 32.0)),
        }
    }
}

impl Add<LocationOrigin> for Location {
    type Output = Location;

    fn add(mut self, rhs: LocationOrigin) -> Self::Output {
        rhs.x.apply(&mut self.x);
        rhs.y.apply(&mut self.y);
        rhs.z.apply(&mut self.z);
        self
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Origin<T> {
    Rel(T),
    Abs(T),
}

impl<T> Origin<T> {
    fn from(value: T, relative: bool) -> Origin<T> {
        if relative {
            Origin::Rel(value)
        } else {
            Origin::Abs(value)
        }
    }
}

impl Origin<f64> {
    pub fn apply(&self, other: &mut f64) {
        match self {
            Origin::Rel(x) => *other += *x,
            Origin::Abs(x) => *other = *x
        }
    }
}

impl Origin<f32> {

    pub fn apply(&self, other: &mut f32) {
        match self {
            Origin::Rel(x) => *other += *x,
            Origin::Abs(x) => *other = *x
        }
    }
}

#[derive(Debug)]
pub struct LocationOrigin {
    pub x: Origin<f64>,
    pub y: Origin<f64>,
    pub z: Origin<f64>,
}

impl LocationOrigin {
    pub fn from(location: Location, x: bool, y: bool, z: bool) -> LocationOrigin {
        LocationOrigin {
            x: Origin::from(location.x, x),
            y: Origin::from(location.y, y),
            z: Origin::from(location.z, z),
        }
    }
}

#[derive(Debug)]
pub struct DirectionOrigin {
    pub yaw: Origin<f32>,
    pub pitch: Origin<f32>,
}

impl DirectionOrigin {
    pub fn from(dir: Direction, yaw: bool, pitch: bool) -> DirectionOrigin {
        DirectionOrigin {
            yaw: Origin::from(dir.yaw, yaw),
            pitch: Origin::from(dir.pitch, pitch),
        }
    }
}

#[derive(Readable, Writable)]
pub struct Direction {
    /// wiki.vg:
    ///yaw is measured in degrees, and does not follow classical trigonometry rules.
    ///The unit circle of yaw on the XZ-plane starts at (0, 1) and turns counterclockwise, with 90 at (-1, 0), 180 at (0,-1) and 270 at (1, 0).
    ///Additionally, yaw is not clamped to between 0 and 360 degrees; any number is valid, including negative numbers and numbers greater than 360.
    pub yaw: f32,
    pub pitch: f32,
}

impl Direction {

    pub fn from(dx: f32, dy: f32, dz: f32) -> Direction {
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        let mut yaw = -dx.atan2(dz) / PI * 180.0;
        if yaw < 0.0 {
            yaw += 360.0
        }
        let pitch = -(dy / r).asin() / PI * 180.0;
        Direction {
            yaw,
            pitch,
        }
    }
}
