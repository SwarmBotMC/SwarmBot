/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::f32::consts::PI;
use std::fmt::{Display, Formatter};
use std::lazy::SyncLazy;
use std::ops::{Add, AddAssign, Index, Mul, MulAssign, Neg, Sub};

use ansi_term::Style;
use itertools::Itertools;
use packets::*;
use packets::read::{ByteReadable, ByteReader};

use packets::write::{ByteWritable, ByteWriter};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::client::pathfind::moves::Change;
use crate::storage::block::{BlockLocation};
use crate::types::Origin::{Abs, Rel};
use crate::client::state::local::inventory::ItemStack;

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
        use ansi_term::Color::*;

        let color = match self.color.unwrap_or_default().as_str() {
            "dark_blue" | "blue" => Blue,
            "dark_aqua" | "aqua" => Cyan,
            "red" | "dark_red" => Red,
            "purple" | "light_purple" => Purple,
            "gold" | "yellow" => Yellow,
            "gray" => White,
            "dark_gray" => Black,
            "green" | "dark_green" => Green,
            "white" => White,
            _ => Black
        };

        let mut res = Style::from(color);

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

        res.paint(self.text).to_string()
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
        static RE: SyncLazy<Regex> = SyncLazy::new(|| {
            Regex::new(r"^#(\S+)\s?(.*)").unwrap()
        });
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
        static RE: SyncLazy<Regex> = SyncLazy::new(|| {
            Regex::new(r"^([A-Za-z_0-9]+) whispers: (.*)").unwrap()
        });

        let text = self.extra.as_ref()?.iter().map(|x| &x.text).join("");

        let captures: Captures = RE.captures(&text)?;

        let player = captures.get(1)?.as_str().to_string();
        let message = captures.get(2)?.as_str().to_string();
        Some(PlayerMessage {
            player,
            message,
        })
    }
    pub fn player_message(&self) -> Option<PlayerMessage> {
        static RE: SyncLazy<Regex> = SyncLazy::new(|| {
            Regex::new(r"^<([A-Za-z_0-9]+)> (.*)").unwrap()
        });

        let text = self.extra.as_ref()?.iter().map(|x| &x.text).join("");

        let captures: Captures = RE.captures(&text)?;

        let player = captures.get(1)?.as_str().to_string();
        let message = captures.get(2)?.as_str().to_string();

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
        Location { x: x.round(), y: y.round(), z: z.round() }
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
    pub id: u16
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemNbt {
    pub ench: Option<Vec<Enchantment>>
}

impl ByteReadable for ItemNbt {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        nbt::from_reader(byte_reader).unwrap()
    }
}

impl ByteWritable for ItemNbt {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        nbt::to_writer(writer, &self, None).unwrap();
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


/// https://wiki.vg/Slot_Data
#[derive(Debug)]
pub struct Slot {
    pub block_id: i16,
    pub item_count: Option<u8>,
    pub item_damage: Option<u16>,
    pub nbt: Option<ItemNbt>,
}

impl From<ItemStack> for Slot {
    fn from(stack: ItemStack) -> Self {
        Self {
            block_id: stack.kind.0 as i16,
            item_count: Some(stack.count),
            item_damage: Some(stack.damage),
            nbt: stack.nbt,
        }
    }
}

impl Slot {
    pub const EMPTY: Slot = {
        Slot {
            block_id: -1,
            item_count: None,
            item_damage: None,
            nbt: None,
        }
    };

    pub fn present(&self) -> bool {
        self.block_id != -1
    }
}

impl ByteWritable for Slot {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        writer.write(self.block_id);

        if self.block_id != -1 {
            writer.write(self.item_count.unwrap());
            writer.write(self.item_damage.unwrap());

            match self.nbt {
                None => writer.write(0_u8),
                Some(nbt) => writer.write(nbt)
            };
        }
    }
}

impl ByteReadable for Slot {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let block_id: i16 = byte_reader.read();

        if block_id != -1 {
            let item_count = byte_reader.read();
            let item_damage = byte_reader.read();

            let first: u8 = byte_reader.read();
            let nbt = (first != 0).then(|| {
                byte_reader.back(1);
                byte_reader.read()
            });

            Slot {
                block_id,
                item_count: Some(item_count),
                item_damage: Some(item_damage),
                nbt,
            }
        } else {
            Slot {
                block_id,
                item_count: None,
                item_damage: None,
                nbt: None,
            }
        }
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
        f.write_fmt(format_args!("[{:.2} {:.2} {:.2}]", self.dx, self.dy, self.dz))
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
        const EPSILON: f64 = 0.01;

        let dx = if self.dx.abs() < 0.5 {0.} else {self.dx};
        let dy = if self.dy.abs() < 0.5 {0.} else {self.dy};
        let dz = if self.dz.abs() < 0.5 {0.} else {self.dz};
        Self{dx,dy,dz}
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
            _ => panic!("invalid index")
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

#[derive(Readable, Writable, Copy, Clone, Default, Debug)]
pub struct Direction {
    /// wiki.vg:
    ///yaw is measured in degrees, and does not follow classical trigonometry rules.
    ///The unit circle of yaw on the XZ-plane starts at (0, 1) and turns counterclockwise, with 90 at (-1, 0), 180 at (0,-1) and 270 at (1, 0).
    ///Additionally, yaw is not clamped to between 0 and 360 degrees; any number is valid, including negative numbers and numbers greater than 360.
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
        Direction {
            yaw,
            pitch,
        }
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
            Dimension::End => "end"
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
            val => panic!("dimension {} is not valid", val)
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

        if x >= LAT_LON_THRESHOLD { x -= LAT_LON_SUB }
        if y >= Y_THRESH { y -= Y_SUB }
        if z >= LAT_LON_THRESHOLD { z -= LAT_LON_SUB }

        Position {
            x,
            y,
            z,
        }
    }
}

impl ByteWritable for Position {
    fn write_to_bytes(self, writer: &mut ByteWriter) {
        let Position { x, y, z } = self;
        let write = ((x as u64 & 0x3FFFFFF) << 38) | ((y as u64 & 0xFFF) << 26) | (z as u64 & 0x3FFFFFF);
        writer.write(write);
    }
}
