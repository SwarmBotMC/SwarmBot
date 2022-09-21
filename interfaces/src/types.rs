use colored::Colorize;
use std::{
    f32::consts::PI,
    fmt::{Debug, Display, Formatter},
    ops::{Add, AddAssign, Index, Mul, MulAssign, Neg, Sub},
};

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use swarm_bot_packets::{
    read::{ByteReadable, ByteReader},
    write::{ByteWritable, ByteWriter},
    *,
};

use crate::types::{
    block_data::{Block, BlockData},
    Origin::{Abs, Rel},
};

pub mod block_data;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatSection {
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub extra: Option<Vec<ChatSection>>,
    pub text: Option<String>,
}

impl Chat {
    pub fn colorize(self) -> String {
        if let Some(extra) = self.extra {
            extra.into_iter().map(|section| section.colorize()).join("")
        } else {
            String::new()
        }
    }
}

impl ChatSection {
    fn colorize(self) -> String {
        use colored::Color::*;

        let color = match self.color.unwrap_or_default().as_str() {
            "dark_blue" | "blue" => Blue,
            "dark_aqua" | "aqua" => Cyan,
            "red" | "dark_red" => Red,
            "purple" | "light_purple" => Magenta,
            "gold" | "yellow" => Yellow,
            "gray" => White,
            "dark_gray" => Black,
            "green" | "dark_green" => Green,
            "white" => White,
            _ => Black,
        };

        let mut res = self.text.color(color);

        if self.bold.unwrap_or_default() {
            res = res.bold();
        }

        if self.italic.unwrap_or_default() {
            res = res.italic();
        }

        if self.underlined.unwrap_or_default() {
            res = res.underline();
        }

        if self.strikethrough.unwrap_or_default() {
            res = res.strikethrough();
        }

        res.to_string()
    }
}

#[derive(Debug)]
pub struct Command {
    pub player: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct PlayerMessage {
    pub player: String,
    pub message: String,
}

impl PlayerMessage {
    pub fn into_cmd(self) -> Option<Command> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^#(\S+)\s?(.*)").unwrap();
        }
        let capture = RE.captures(&self.message)?;

        let command = capture.get(1)?.as_str().to_string();
        let args = capture.get(2)?.as_str().to_string();

        let args = if args.is_empty() {
            Vec::new()
        } else {
            args.split(' ').map(|x| x.to_string()).collect()
        };

        Some(Command {
            player: self.player,
            command,
            args,
        })
    }
}

impl Chat {
    pub fn player_dm(&self) -> Option<PlayerMessage> {
        lazy_static::lazy_static! {
            static ref RE: Regex =Regex::new(r"^([A-Za-z_0-9]+) whispers: (.*)").unwrap();
        }

        let text = self.extra.as_ref()?.iter().map(|x| &x.text).join("");

        let captures: Captures = RE.captures(&text)?;

        let player = captures.get(1)?.as_str().to_string();
        let message = captures.get(2)?.as_str().to_string();
        Some(PlayerMessage { player, message })
    }
    pub fn player_message(&self) -> Option<PlayerMessage> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^<([A-Za-z_0-9]+)> (.*)").unwrap();
        }

        let text = self.extra.as_ref()?.iter().map(|x| &x.text).join("");

        let captures: Captures = RE.captures(&text)?;

        let player = captures.get(1)?.as_str().to_string();
        let message = captures.get(2)?.as_str().to_string();

        Some(PlayerMessage { player, message })
    }
}

impl ByteReadable for Chat {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let string: String = byte_reader.read();
        let json: Chat = serde_json::from_str(&string).unwrap();
        json
    }
}

#[derive(Writable, Readable, Debug, Copy, Clone, Default, PartialEq)]
pub struct LocationFloat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<LocationFloat> for Location {
    fn from(loc: LocationFloat) -> Self {
        Self {
            x: loc.x as f64,
            y: loc.y as f64,
            z: loc.z as f64,
        }
    }
}

impl Add<Displacement> for Location {
    type Output = Location;

    fn add(self, rhs: Displacement) -> Self::Output {
        Location {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
            z: self.z + rhs.dz,
        }
    }
}

impl Location {
    pub const fn new(x: f64, y: f64, z: f64) -> Location {
        Location { x, y, z }
    }
}

impl From<Location> for BlockLocation {
    fn from(location: Location) -> Self {
        let Location { x, y, z } = location;
        BlockLocation::from_flts(x, y, z)
    }
}

impl Sub<Location> for Location {
    type Output = Displacement;

    fn sub(self, rhs: Location) -> Self::Output {
        Displacement {
            dx: self.x - rhs.x,
            dy: self.y - rhs.y,
            dz: self.z - rhs.z,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:.2} {:.2} {:.2}]", self.x, self.y, self.z))
    }
}

#[derive(Writable, Readable, Debug, Copy, Clone, Default)]
pub struct Displacement {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Enchantment {
    pub lvl: u16,
    pub id: u16,
}

impl Enchantment {
    pub fn efficiency(self) -> Option<u16> {
        if self.id == 32 {
            Some(self.lvl)
        } else {
            None
        }
    }
}

pub struct ShortVec<T>(pub Vec<T>);

impl<T: ByteReadable> ByteReadable for ShortVec<T> {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let length: u16 = byte_reader.read();
        let length = length as usize;
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(byte_reader.read());
        }
        ShortVec(vec)
    }
}

pub struct Change {
    pub dx: i32,
    pub dy: i16,
    pub dz: i32,
}

impl Change {
    pub fn new(dx: i32, dy: i16, dz: i32) -> Change {
        Change { dx, dy, dz }
    }
}

impl From<Change> for Displacement {
    fn from(change: Change) -> Self {
        Self {
            dx: change.dx as f64,
            dy: change.dy as f64,
            dz: change.dz as f64,
        }
    }
}

impl Display for Displacement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{:.2} {:.2} {:.2}]",
            self.dx, self.dy, self.dz
        ))
    }
}

impl Neg for Displacement {
    type Output = Displacement;

    fn neg(self) -> Self::Output {
        self * (-1.0)
    }
}

impl Sub for Displacement {
    type Output = Displacement;

    fn sub(self, rhs: Self) -> Self::Output {
        let rhs = rhs * (-1.0);
        self + rhs
    }
}

impl Add for Displacement {
    type Output = Displacement;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
            dz: self.dz + rhs.dz,
        }
    }
}

impl Displacement {
    pub const EYE_HEIGHT: Displacement = Displacement::new(0., 1.6, 0.);
    pub const EPSILON_Y: Displacement = Displacement::new(0., 0.01, 0.);

    pub const fn new(dx: f64, dy: f64, dz: f64) -> Displacement {
        Displacement { dx, dy, dz }
    }

    pub fn zero_if_reachable(&self) -> Displacement {
        let dx = if self.dx.abs() < 0.5 { 0. } else { self.dx };
        let dy = if self.dy.abs() < 0.5 { 0. } else { self.dy };
        let dz = if self.dz.abs() < 0.5 { 0. } else { self.dz };
        Self { dx, dy, dz }
    }

    pub fn make_dy(&self, dy: f64) -> Displacement {
        Self {
            dx: self.dx,
            dy,
            dz: self.dz,
        }
    }

    pub fn mag(&self) -> f64 {
        self.mag2().sqrt()
    }

    pub fn dot(&self, other: Displacement) -> f64 {
        self.dx * other.dx + self.dy * other.dy + self.dz * other.dz
    }

    pub fn reflect(&self, normal: Displacement) -> Displacement {
        let rhs = normal * 2.0 * (self.dot(normal));
        *self - rhs
    }

    pub fn normalize(self) -> Displacement {
        let mag = self.mag();
        if mag == 0. {
            // we can't normalize 0-length
            self
        } else {
            let mult = 1.0 / mag;
            self * mult
        }
    }

    pub fn mag2(&self) -> f64 {
        let Displacement { dx, dy, dz } = *self;
        dx * dx + dy * dy + dz * dz
    }

    pub fn cross(&self, other: Displacement) -> Displacement {
        let dx = self[1] * other[2] - self[2] * other[1];
        let dy = self[2] * other[0] - self[0] * other[2];
        let dz = self[0] * other[1] - self[1] * other[0];
        Displacement::new(dx, dy, dz)
    }
    pub fn has_length(&self) -> bool {
        self.dx != 0.0 || self.dy != 0.0 || self.dz != 0.0
    }
}

impl MulAssign<f64> for Displacement {
    fn mul_assign(&mut self, rhs: f64) {
        self.dx *= rhs;
        self.dy *= rhs;
        self.dz *= rhs;
    }
}

impl Mul<f64> for Displacement {
    type Output = Displacement;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
            dz: self.dz * rhs,
        }
    }
}

impl Index<usize> for Displacement {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.dx,
            1 => &self.dy,
            2 => &self.dz,
            _ => panic!("invalid index"),
        }
    }
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
            Rel(value)
        } else {
            Abs(value)
        }
    }
}

impl Origin<f64> {
    pub fn apply(&self, other: &mut f64) {
        match self {
            Rel(x) => *other += *x,
            Abs(x) => *other = *x,
        }
    }
}

impl Origin<f32> {
    pub fn apply(&self, other: &mut f32) {
        match self {
            Rel(x) => *other += *x,
            Abs(x) => *other = *x,
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

#[derive(Readable, Writable, Copy, Clone, Default, Debug)]
pub struct Direction {
    /// wiki.vg:
    /// yaw is measured in degrees, and does not follow classical trigonometry
    /// rules. The unit circle of yaw on the XZ-plane starts at (0, 1) and
    /// turns counterclockwise, with 90 at (-1, 0), 180 at (0,-1) and 270 at (1,
    /// 0). Additionally, yaw is not clamped to between 0 and 360 degrees;
    /// any number is valid, including negative numbers and numbers greater than
    /// 360.
    pub yaw: f32,
    pub pitch: f32,
}

impl Direction {
    pub const DOWN: Direction = Direction {
        yaw: 90.,
        pitch: 90.,
    };

    pub fn unit_vector(&self) -> Displacement {
        let pitch = self.pitch.to_radians();
        let yaw = self.yaw.to_radians();

        let x = -(pitch).to_radians().cos() * (yaw).sin();
        let y = -(pitch).sin();
        let z = (pitch).cos() * (yaw).cos();

        Displacement::new(x as f64, y as f64, z as f64)
    }

    pub fn horizontal(&self) -> Direction {
        let mut res = *self;
        res.pitch = 0.0;
        res
    }
}

impl From<Displacement> for Direction {
    fn from(displacement: Displacement) -> Self {
        let Displacement { dx, dy, dz } = displacement;
        let (dx, dy, dz) = (dx as f32, dy as f32, dz as f32);
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        let mut yaw = -dx.atan2(dz) / PI * 180.0;

        if yaw < 0.0 {
            yaw += 360.0
        }

        const EPSILON: f32 = 0.1;

        if yaw.abs() < EPSILON {
            yaw = 0.0;
        }
        let pitch = -(dy / r).asin() / PI * 180.0;
        Direction { yaw, pitch }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Dimension {
    Nether,
    Overworld,
    End,
}

impl Display for Dimension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            Dimension::Nether => "nether",
            Dimension::Overworld => "overworld",
            Dimension::End => "end",
        };
        f.write_str(to_write)
    }
}

impl ByteReadable for Dimension {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        use Dimension::*;
        let val: i32 = byte_reader.read();
        match val {
            -1 => Nether,
            0 => Overworld,
            1 => End,
            val => panic!("dimension {} is not valid", val),
        }
    }
}

pub type Position = BlockLocation;

impl ByteReadable for Position {
    ///
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let val: u64 = byte_reader.read();

        let mut x = (val >> 38) as i32;
        let mut y = ((val >> 26) & 0xFFF) as i16;
        let mut z = (val << 38 >> 38) as i32;

        const LAT_LON_THRESHOLD: i32 = 1 << 25;
        const LAT_LON_SUB: i32 = 1 << 26;

        const Y_THRESH: i16 = 1 << 11;
        const Y_SUB: i16 = 1 << 12;

        if x >= LAT_LON_THRESHOLD {
            x -= LAT_LON_SUB
        }
        if y >= Y_THRESH {
            y -= Y_SUB
        }
        if z >= LAT_LON_THRESHOLD {
            z -= LAT_LON_SUB
        }

        Position { x, y, z }
    }
}

impl ByteWritable for Position {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        let Position { x, y, z } = self;
        let write =
            ((x as u64 & 0x3FFFFFF) << 38) | ((y as u64 & 0xFFF) << 26) | (z as u64 & 0x3FFFFFF);
        writer.write(write);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Selection2D {
    pub from: BlockLocation2D,
    pub to: BlockLocation2D,
}

impl Selection2D {
    /// Normalize so that the **from** coordinate is always smaller than the
    /// **to** coord.
    pub fn normalize(self) -> Self {
        let min_x = self.from.x.min(self.to.x);
        let min_z = self.from.z.min(self.to.z);

        let max_x = self.from.x.max(self.to.x);
        let max_z = self.from.z.max(self.to.z);

        Selection2D {
            from: BlockLocation2D::new(min_x, min_z),
            to: BlockLocation2D::new(max_x, max_z),
        }
    }
}

#[derive(Writable, Readable, Debug, Copy, Clone, Default, PartialEq)]
pub struct Location {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Location {
    pub fn sub_y(&self, dy: f64) -> Location {
        Location::new(self.x, self.y - dy, self.z)
    }

    pub fn round(&self) -> Location {
        let &Location { x, y, z } = self;
        Location {
            x: x.round(),
            y: y.round(),
            z: z.round(),
        }
    }

    pub fn add_y(&self, dy: f64) -> Location {
        Location::new(self.x, self.y + dy, self.z)
    }
}

impl Sub<Displacement> for Location {
    type Output = Location;

    fn sub(self, rhs: Displacement) -> Self::Output {
        let Displacement { dx, dy, dz } = rhs;
        Self {
            x: self.x - dx,
            y: self.y - dy,
            z: self.z - dz,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ChunkLocation(pub i32, pub i32);

impl From<BlockLocation> for ChunkLocation {
    fn from(loc: BlockLocation) -> Self {
        Self(loc.x >> 4, loc.z >> 4)
    }
}

impl From<Location> for ChunkLocation {
    fn from(loc: Location) -> Self {
        let block_loc = BlockLocation::from(loc);
        Self::from(block_loc)
    }
}

/// A block location stored by (x,z) = i32, y = i16. y is signed to preserve
/// compatibility with 1.17, where the world height can be much higher and goes
/// to negative values.
#[derive(
    Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Default, Serialize, Deserialize,
)]
pub struct BlockLocation {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl From<BlockLocation> for BlockLocation2D {
    fn from(loc: BlockLocation) -> Self {
        Self { x: loc.x, z: loc.z }
    }
}

impl From<BlockLocation2D> for BlockLocation {
    fn from(loc: BlockLocation2D) -> Self {
        Self {
            x: loc.x,
            y: 0,
            z: loc.z,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockLocation2D {
    pub x: i32,
    pub z: i32,
}

impl BlockLocation2D {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    pub fn dist2(self, other: BlockLocation2D) -> u64 {
        // TODO: potential bug here with unsigned abs
        let dx = self.x.abs_diff(other.x) as u64;
        let dz = self.z.abs_diff(other.z) as u64;
        dx * dx + dz * dz
    }
}

impl From<Change> for BlockLocation {
    fn from(change: Change) -> Self {
        Self {
            x: change.dx,
            y: change.dy,
            z: change.dz,
        }
    }
}

impl Add for BlockLocation {
    type Output = BlockLocation;

    fn add(self, rhs: Self) -> Self::Output {
        let BlockLocation { x, y, z } = self;
        BlockLocation::new(x + rhs.x, y + rhs.y, z + rhs.z)
    }
}

impl BlockLocation {
    pub fn new(x: i32, y: i16, z: i32) -> BlockLocation {
        BlockLocation { x, y, z }
    }

    pub fn faces(self) -> [Location; 6] {
        const DISPLACEMENTS: [Displacement; 6] = {
            let a = Displacement::new(0.5, 0.0, 0.5);
            let b = Displacement::new(0.5, 1.0, 0.5);

            let c = Displacement::new(0.5, 0.5, 0.0);
            let d = Displacement::new(0.5, 0.5, 1.0);

            let e = Displacement::new(0.0, 0.5, 0.5);
            let f = Displacement::new(1.0, 0.5, 0.5);

            [a, b, c, d, e, f]
        };

        let lowest = Location::new(self.x as f64, self.y as f64, self.z as f64);
        let mut res = [Location::default(); 6];
        for i in 0..6 {
            res[i] = lowest + DISPLACEMENTS[i]
        }
        res
    }

    pub fn below(&self) -> BlockLocation {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }

    pub fn above(&self) -> BlockLocation {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn get(&self, idx: usize) -> i32 {
        match idx {
            0 => self.x,
            1 => self.y as i32,
            2 => self.z,
            _ => panic!("invalid index for block location"),
        }
    }

    pub fn set(&mut self, idx: usize, value: i32) {
        match idx {
            0 => self.x = value,
            1 => self.y = value as i16,
            2 => self.z = value,
            _ => panic!("invalid index for block location"),
        }
    }

    pub fn from_flts(x: impl num::Float, y: impl num::Float, z: impl num::Float) -> BlockLocation {
        let x = num::cast(x.floor()).unwrap();
        let y = num::cast(y.floor()).unwrap_or(-100); // TODO: change.. however, this is the best for an invalid number right now.
        let z = num::cast(z.floor()).unwrap();
        BlockLocation::new(x, y, z)
    }

    pub fn add_y(&self, dy: i16) -> BlockLocation {
        let &BlockLocation { x, y, z } = self;
        Self { x, y: y + dy, z }
    }

    pub fn center_bottom(&self) -> Location {
        Location {
            x: self.x as f64 + 0.5,
            y: self.y as f64,
            z: self.z as f64 + 0.5,
        }
    }

    pub fn true_center(&self) -> Location {
        Location {
            x: self.x as f64 + 0.5,
            y: self.y as f64 + 0.5,
            z: self.z as f64 + 0.5,
        }
    }
}

impl BlockLocation {
    pub fn dist2(&self, other: BlockLocation) -> f64 {
        let (dx, dy, dz) = self.abs_dif(other);
        let (dx, dy, dz) = (dx as f64, dy as f64, dz as f64);
        dx * dx + dy * dy + dz * dz
    }

    pub fn abs_dif(&self, other: BlockLocation) -> (u32, u16, u32) {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        let dz = self.z.abs_diff(other.z);
        (dx, dy, dz)
    }

    pub fn manhatten(&self, other: BlockLocation) -> u64 {
        let (dx, dy, dz) = self.abs_dif(other);
        let (dx, dy, dz) = (dx as u64, dy as u64, dz as u64);
        dx + dy + dz
    }

    pub fn dist(&self, other: BlockLocation) -> f64 {
        (self.dist2(other) as f64).sqrt()
    }
}

impl Display for BlockLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}, {}]", self.x, self.y, self.z))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BlockApprox {
    Realized(BlockState),
    Estimate(SimpleType),
}

impl BlockApprox {
    pub const AIR: BlockApprox = BlockApprox::Estimate(SimpleType::WalkThrough);

    pub fn s_type(&self) -> SimpleType {
        match self {
            BlockApprox::Realized(x) => x.simple_type(),
            BlockApprox::Estimate(x) => *x,
        }
    }

    pub fn as_real(&self) -> BlockState {
        match self {
            BlockApprox::Realized(inner) => *inner,
            _ => panic!("was not realized"),
        }
    }

    pub fn is_solid(&self) -> bool {
        self.s_type() == SimpleType::Solid
    }

    pub fn is_walkable(&self) -> bool {
        self.s_type() == SimpleType::WalkThrough
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SimpleType {
    Solid,
    Water,
    Avoid,
    WalkThrough,
}

impl SimpleType {
    pub fn id(&self) -> u8 {
        match self {
            SimpleType::Solid => 0,
            SimpleType::Water => 1,
            SimpleType::Avoid => 2,
            SimpleType::WalkThrough => 3,
        }
    }
}

impl From<u8> for SimpleType {
    fn from(id: u8) -> Self {
        match id {
            0 => SimpleType::Solid,
            1 => SimpleType::Water,
            2 => SimpleType::Avoid,
            3 => SimpleType::WalkThrough,
            _ => panic!("invalid id"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct BlockKind(pub u32);

impl From<u32> for BlockKind {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl BlockKind {
    pub const DEFAULT_SLIP: f64 = 0.6;
    pub const LADDER: BlockKind = BlockKind(65);
    pub const LEAVES: BlockKind = BlockKind(18);
    pub const FLOWING_WATER: BlockKind = BlockKind(8);
    pub const STONE: BlockKind = BlockKind(1);
    pub const DIRT: BlockKind = BlockKind(3);
    pub const GLASS: BlockKind = BlockKind(20);

    #[inline]
    pub fn id(self) -> u32 {
        self.0
    }

    pub fn hardness(&self, blocks: &BlockData) -> Option<f64> {
        let block = blocks
            .by_id(self.0)
            .unwrap_or_else(|| panic!("no block for id {}", self.0));
        block.hardness
    }

    pub fn data<'a>(&self, blocks: &'a BlockData) -> &'a Block {
        blocks
            .by_id(self.0)
            .unwrap_or_else(|| panic!("no block for id {}", self.0))
    }

    pub fn throw_away_block(self) -> bool {
        // cobblestone
        matches!(self.id(), 4)
    }

    pub fn mineable(&self, blocks: &BlockData) -> bool {
        // we can't mine air
        if self.0 == 0 {
            return false;
        }

        match self.hardness(blocks) {
            None => false,
            Some(val) => val < 100.0,
        }
    }

    pub fn slip(&self) -> f64 {
        match self.0 {
            266 => 0.989,           // blue ice
            79 | 174 | 212 => 0.98, // ice, packed ice, or frosted ice
            37 => 0.8,              // slime block
            _ => Self::DEFAULT_SLIP,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
#[repr(transparent)]
pub struct BlockState(pub u32);

impl Debug for BlockState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.0 >> 4, self.0 % 16))
    }
}

impl BlockState {
    pub const AIR: BlockState = BlockState(0);
    pub const WATER: BlockState = BlockState(9);
    pub const STONE: BlockState = BlockState(16);

    pub fn from(id: u32, data: u16) -> BlockState {
        BlockState((id << 4) + data as u32)
    }

    pub fn id(&self) -> u32 {
        self.0 >> 4
    }

    pub fn kind(&self) -> BlockKind {
        BlockKind(self.id())
    }

    pub fn simple_type(&self) -> SimpleType {
        if self.full_block() {
            return SimpleType::Solid;
        }

        if self.is_water() {
            return SimpleType::Water;
        }

        if self.walk_through() {
            return SimpleType::WalkThrough;
        }

        SimpleType::Avoid
    }

    pub fn metadata(&self) -> u8 {
        (self.0 & 0b1111) as u8
    }

    pub fn full_block(&self) -> bool {
        // consider 54 |
        matches!(self.id(),
            1..=5 |7 | 12..=25 | 29 | 33 |35 | 41 ..=43 | 45..=49 | 52 | 56..=58 | 60..=62 | 73 | 74 |
            78..=80| // snow, ice
            82| // clay
            84|86|87|89|91|95|
            97| // TODO: avoid this is a monster egg
            98..=100|
            // TODO: account panes
            103|110|112|118|121|123..=125|
            129|133|137..=138|155|159|161|162|
            165| // TODO: slime block special fall logic
            166|
            168..=170| // TODO: special haybale logic
            172..=174|
            179|181|199..=202|
            204|206|208..=212|214..=255

        )
    }

    pub fn is_water(&self) -> bool {
        matches!(
            self.id(),
            8 | 9 | 65 // ladder ... this is VERY jank
        )
    }

    pub fn walk_through(&self) -> bool {
        self.is_water() || self.no_motion_effect()
    }

    pub fn no_motion_effect(&self) -> bool {
        matches!(
            self.id(),
            0| // air
            6|// sapling
            27|28| //  rail
            31| // grass/fern/dead shrub
            38|37|// flower
            39|40| //mushroom
            50|//torch
            59|// wheat
            66|68|69|70|72|75|76|77|83|
            90| // portal
            104|105|106|
            115|119|
            175..=177
        )
    }
}
