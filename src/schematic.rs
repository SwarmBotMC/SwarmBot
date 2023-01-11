use std::io::Read;

use anyhow::Context;
use interfaces::types::{BlockLocation, BlockState};
use serde::{Deserialize, Serialize};

/// The `WorldEdit` schematic format
/// <https://minecraft.fandom.com/wiki/Schematic_file_format>
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Schematic {
    pub width: i16,
    pub height: i16,
    pub length: i16,
    materials: String,
    blocks: Vec<i8>,
    add_blocks: Option<Vec<i8>>,
    data: Vec<i8>,
    w_e_origin_x: Option<i32>,
    w_e_origin_y: Option<i32>,
    w_e_origin_z: Option<i32>,
    w_e_offset_x: Option<i32>,
    w_e_offset_y: Option<i32>,
    w_e_offset_z: Option<i32>,
}

impl Schematic {
    #[allow(unused)]
    pub const fn volume(&self) -> u64 {
        let v = (self.width as i64) * (self.height as i64) * (self.length as i64);
        v.abs_diff(0)
    }

    #[allow(unused)]
    pub fn load(reader: &mut impl Read) -> anyhow::Result<Self> {
        let res: Result<Self, _> =
            nbt::from_gzip_reader(reader).context("could not load schematic");

        res
    }

    #[allow(unused)]
    pub fn is_valid(&self) -> bool {
        self.volume() == self.blocks.len() as u64
    }

    #[allow(unused)]
    pub fn origin(&self) -> Option<BlockLocation> {
        match (self.w_e_origin_x, self.w_e_origin_y, self.w_e_origin_z) {
            (Some(x), Some(y), Some(z)) => Some(BlockLocation::new(x, y as i16, z)),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn offset(&self) -> Option<BlockLocation> {
        match (self.w_e_offset_x, self.w_e_offset_y, self.w_e_offset_z) {
            (Some(x), Some(y), Some(z)) => Some(BlockLocation::new(x, y as i16, z)),
            _ => None,
        }
    }

    #[allow(unused, clippy::unwrap_used)]
    pub fn width(&self) -> u64 {
        u64::try_from(self.width).unwrap()
    }

    #[allow(unused, clippy::unwrap_used)]
    pub fn height(&self) -> u64 {
        u64::try_from(self.width).unwrap()
    }

    #[allow(unused, clippy::unwrap_used)]
    pub fn length(&self) -> u64 {
        u64::try_from(self.length).unwrap()
    }

    #[allow(unused, clippy::unwrap_used, clippy::indexing_slicing)]
    pub fn blocks(&self) -> impl Iterator<Item = (BlockLocation, BlockState)> + '_ {
        let origin = self.origin().unwrap_or_default();

        (0..self.volume()).map(move |idx| {
            let x = idx % self.width();

            let leftover = idx / self.width();
            let z = leftover % self.length();

            let y = leftover / self.length();

            let location = BlockLocation::new(x as i32, y as i16, z as i32) + origin;

            let id = self.blocks[idx as usize].abs_diff(0);
            let data = self.data[idx as usize].abs_diff(0);
            let state = BlockState::from(u32::from(id), u16::from(data));

            (location, state)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::OpenOptions};

    use interfaces::types::BlockLocation;
    use more_asserts::*;

    use crate::schematic::Schematic;

    #[test]
    fn test_load() {
        let mut reader = OpenOptions::new()
            .read(true)
            .open("test-data/parkour.schematic")
            .unwrap();

        let schematic = Schematic::load(&mut reader).unwrap();

        assert!(schematic.is_valid());

        let origin = schematic.origin().unwrap_or_default();

        let mut map = HashMap::new();
        for (loc, state) in schematic.blocks() {
            assert_ge!(loc.x, origin.x);
            assert_lt!(loc.x, origin.x + schematic.width as i32);

            assert_ge!(loc.y, origin.y);
            assert_lt!(loc.y, origin.y + schematic.height);

            assert_ge!(loc.z, origin.z);
            assert_lt!(loc.z, origin.z + schematic.length as i32);
            map.insert(loc, state);
        }

        let stained_glass = map[&BlockLocation::new(-162, 81, -357)];
        assert_eq!(stained_glass.id(), 95);
    }
}
