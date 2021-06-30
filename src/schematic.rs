/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/30/21, 1:15 PM
 */

use serde::{Deserialize, Serialize};

use crate::storage::block::{BlockLocation, BlockState};
use std::io::{Read};

/// https://minecraft.fandom.com/wiki/Schematic_file_format
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Schematic {
    width: i16,
    height: i16,
    length: i16,
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

    // #[serde(rename = "x")]
    // x: i32,
    //
    // #[serde(rename = "y")]
    // y: i32,
    //
    // #[serde(rename = "z")]
    // z: i32
}

impl Schematic {
    pub fn volume(&self) -> i16 {
        self.width * self.height * self.length
    }

    pub fn load(reader: &mut impl Read) -> Schematic {
        let res: Result<Schematic, _> = nbt::from_gzip_reader(reader);
        res.unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.volume() == self.blocks.len() as i16
    }

    pub fn origin(&self) -> Option<BlockLocation> {
        match (self.w_e_origin_x, self.w_e_origin_y, self.w_e_origin_z) {
            (Some(x), Some(y), Some(z)) => {
                Some(BlockLocation::new(x, y as i16, z))
            },
            _ => None
        }
    }

    pub fn offset(&self) -> Option<BlockLocation> {
        match (self.w_e_offset_x, self.w_e_offset_y, self.w_e_offset_z) {
            (Some(x), Some(y), Some(z)) => {
                Some(BlockLocation::new(x, y as i16, z))
            },
            _ => None
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item=(BlockLocation, BlockState)> + '_ {

        let d_loc = self.origin().unwrap_or_default();

        (0..self.volume())
            .map(move |idx| {
                let x = idx % self.width;

                let leftover = idx / self.width;
                let z = leftover % self.length;

                let y = leftover / self.length;

                let location = BlockLocation::new(x as i32, y, z as i32) + d_loc;

                let id = self.blocks[idx as usize];
                let data = self.data[idx as usize];
                let state = BlockState::from(id as u32, data as u32);

                (location, state)
            })
    }

    // pub fn trim(self) -> Schematic {
    //     let mut min = BlockLocation::new(self.width as i32, self.height, self.length as i32);
    //     let mut max = BlockLocation::default();
    //     for (location, state) in self.blocks() {
    //         if state.id() != 0 {
    //             for i in 0..3 {
    //                 if location.get(i) < min.get(i) {
    //                     min.set(i, location.get(i));
    //                 }
    //
    //                 if location.get(i) > max.get(i) {
    //                     max.set(i, location.get(i));
    //                 }
    //             }
    //         };
    //     };
    //
    //     let width = ((max.x - min.x) + 1) as i16;
    //     let length = ((max.z - min.z) + 1) as i16;
    //     let height = (max.y - min.y) + 1;
    //
    //     let new_volume = (width * length * height) as usize;
    //
    //     let mut blocks = Vec::with_capacity(new_volume);
    //     let mut data = Vec::with_capacity(new_volume);
    //
    //     for x in min.x..=max.x {
    //         for z in min.z..=max.z {
    //             for y in min.y..=max.y {
    //                 let x = x as i16;
    //                 let z = z as i16;
    //                 let index = (x + (z * self.width) + (y * self.width * self.length)) as usize;
    //                 blocks.push(self.blocks[index]);
    //                 data.push(self.data[index]);
    //             }
    //         }
    //     }
    //
    //     Schematic {
    //         width,
    //         height,
    //         length,
    //         materials: self.materials,
    //         blocks,
    //         add_blocks: self.add_blocks,
    //         data,
    //         w_e_origin_x: self.w_e_origin_x,
    //         w_e_origin_y: self.w_e_origin_y,
    //         w_e_origin_z: self.w_e_origin_z,
    //         w_e_offset_x: self.w_e_offset_x,
    //         w_e_offset_y: self.w_e_offset_y,
    //         w_e_offset_z: self.w_e_offset_z,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use crate::schematic::Schematic;
    use crate::storage::block::BlockLocation;
    use std::collections::HashMap;
    use more_asserts::*;

    #[test]
    fn test_load() {

        let mut reader = OpenOptions::new()
            .read(true)
            .open("test-data/parkour.schematic")
            .unwrap();

        let schematic = Schematic::load(&mut reader);

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
