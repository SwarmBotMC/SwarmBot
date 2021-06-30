/*
 * Copyright (c) 2021 Minecraft IGN RevolutionNow - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by RevolutionNow <Xy8I7.Kn1RzH0@gmail.com>, 6/29/21, 8:16 PM
 */

use std::collections::HashMap;

use crate::storage::block::{BlockApprox, BlockLocation, BlockState, SimpleType};

const SECTION_ELEMENTS: usize = 16 * 16 * 16;
const BITS_PER_ENUM: usize = 2;
const SECTION_BYTES: usize = SECTION_ELEMENTS * BITS_PER_ENUM / 8;

const ONE_MASK: u64 = !0;

#[derive(Default)]
pub struct HighMemoryChunkSection {
    pub palette: Palette,
}

impl HighMemoryChunkSection {
    pub fn new(palette: Palette) -> Self {
        HighMemoryChunkSection {
            palette
        }
    }
}

#[repr(packed)]
pub struct LowMemoryChunkSection {
    storage: [u8; SECTION_BYTES],
}

impl Default for LowMemoryChunkSection {
    fn default() -> Self {
        Self {
            storage: [0; SECTION_BYTES]
        }
    }
}

pub fn bits_needed(mut number: usize) -> u8 {
    let mut bits = 0_u8;
    while number != 0 {
        number /= 2;
        bits += 1;
    }
    bits
}

impl LowMemoryChunkSection {
    fn get_simple_type(&self, x: u8, y: u8, z: u8) -> SimpleType {
        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;

        // 2 bits per block
        let idx = block_number >> 2;
        let offset = block_number - (idx << 2);

        let mut res = self.storage[idx];
        res >>= offset;
        res &= 0b11;

        SimpleType::from(res)
    }

    fn set_simple_type(&mut self, x: u8, y: u8, z: u8, input: SimpleType) {
        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;

        // 2 bits per block
        let idx = block_number >> 2;
        let offset = block_number - (idx << 2);

        let mut block = self.storage[idx];

        let id = input.id();

        let zero_out = !(0b11 << offset);
        block &= zero_out;

        block |= id << offset;

        self.storage[idx] = block;
    }
}

pub struct ChunkData<T> {
    pub sections: [Option<T>; 16],
}

impl ChunkData<HighMemoryChunkSection> {
    pub fn select(&'a self, mut selector: impl FnMut(BlockState) -> bool + 'a) -> impl Iterator<Item=usize> + 'a {
        self.sections.iter().enumerate()
            .filter_map(|(chunk_idx, section)| section.as_ref().map(|sec| (chunk_idx << 12, sec)))
            .flat_map(|(idx_start, section)| {
                IntoIterator::into_iter(section.palette.all_states()).enumerate().map(move |(idx, state)| (idx_start + idx, state))
            })
            .filter(move |(_, state)| {
                selector(*state)
            })
            .map(|(idx, _)| idx)
    }
}

const SECTION_HEIGHT: usize = 16;
const SECTION_WIDTH: usize = 16;

pub struct Palette {
    bits_per_block: u8,
    id_to_state: Option<Vec<BlockState>>,

    /// invariant: must always be at bits_per_block * 4096 / 64 ... if 4 we have 256 ... if we have 1 we get 64
    storage: Vec<u64>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            /// the smallest bpb
            bits_per_block: 1,
            id_to_state: Some(vec![BlockState::AIR]),
            storage: vec![0; 64],
        }
    }
}

impl Palette {
    pub fn direct(storage: Vec<u64>) -> Palette {
        Palette {
            bits_per_block: 13,
            id_to_state: None,
            storage,
        }
    }

    pub fn indirect(bits_per_block: u8, id_to_state: Vec<BlockState>, storage: Vec<u64>) -> Palette {
        assert!(bits_per_block >= 4);
        assert!(bits_per_block <= 8);
        assert_eq!(storage.len(), 4096 / 64 * bits_per_block as usize);
        Palette {
            bits_per_block,
            id_to_state: Some(id_to_state),
            storage,
        }
    }

    pub fn all_states(&self) -> [BlockState; 4096] {
        let mut res = [BlockState::AIR; 4096];
        (0..4096).for_each(|i| res[i] = self.get_block_by_idx(i));
        res
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, state: BlockState) {
        let value = match self.id_to_state.as_mut() {
            None => state.0,
            Some(map) => {

                // we only have to modify the map if we do not have the state
                let value = map.iter().position(|&r| r == state);
                match value {
                    None => {
                        let new_len = map.len() + 1;
                        let required_bits = bits_needed(new_len);

                        map.push(state);

                        if required_bits > self.bits_per_block {
                            let (required_bits, reverse_map) = if required_bits <= 8 {
                                let reverse_map: HashMap<_, _> = map.iter().enumerate().map(|(k, v)| (*v, k)).collect();

                                (required_bits.max(4), Some(reverse_map))
                            } else {
                                self.id_to_state = None;

                                (13, None)
                            };


                            // we have to recreate the palette
                            let states = self.all_states();

                            // update bits per block
                            self.bits_per_block = required_bits;

                            let required_bits = required_bits as usize;

                            let indv_value_mask = (1 << required_bits) - 1;


                            assert!(required_bits >= 4);
                            let new_data_size = 4096 * (required_bits as usize) / 64;
                            let mut storage = vec![0_u64; new_data_size];

                            for (block_number, state) in IntoIterator::into_iter(states).enumerate() {
                                let start_long = (block_number * required_bits) / 64;
                                let start_offset = (block_number * required_bits) % 64;
                                let end_long = ((block_number + 1) * required_bits - 1) / 64;

                                let value = match reverse_map.as_ref() {
                                    None => state.0 as u64,
                                    Some(reverse_map) => *reverse_map.get(&state).unwrap() as u64
                                };

                                let value = value & indv_value_mask;

                                storage[start_long] |= value << start_offset;

                                if start_long != end_long {
                                    storage[end_long] = value >> (64 - start_offset);
                                }
                            }

                            // TODO: culprit?
                            self.storage = storage;
                            return;
                        } else {
                            (new_len - 1) as u32
                        }
                    }
                    Some(value) => {
                        value as u32
                    }
                }
            }
        };


        let value = value as u64;

        let indv_value_mask = (1 << self.bits_per_block) - 1;

        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;
        let bits_per_block = self.bits_per_block as usize;
        let start_long = (block_number * bits_per_block) / 64;
        let start_offset = (block_number * bits_per_block) % 64;
        let end_long = ((block_number + 1) * bits_per_block - 1) / 64;

        // zero out TODO: thread 'main' panicked at 'index out of bounds: the len is 0 but the index is 0', below...
        self.storage[start_long] &= !(indv_value_mask << start_offset);

        // place
        self.storage[start_long] |= value << start_offset;

        if start_long != end_long {

            // zero out
            self.storage[end_long] &= !(indv_value_mask >> (64 - start_offset));

            // set value
            self.storage[end_long] |= value >> (64 - start_offset);
        }
    }

    fn get_block_by_idx(&self, block_number: usize) -> BlockState {
        let data_arr = &self.storage;

        let bits_per_block = self.bits_per_block as usize;

        let indv_value_mask = (1 << bits_per_block) - 1;

        let start_long = (block_number * bits_per_block) / 64;
        let start_offset = (block_number * bits_per_block) % 64;
        let end_long = ((block_number + 1) * bits_per_block - 1) / 64;

        let data = if start_long == end_long {
            (data_arr[start_long] >> start_offset) as u32
        } else {
            let end_offset = 64 - start_offset;
            (data_arr[start_long] >> start_offset | data_arr[end_long] << end_offset) as u32
        };

        let data = data & indv_value_mask;

        match &self.id_to_state {
            None => BlockState(data),
            Some(map) => {
                *map.get(data as usize).expect("internal chunk error getting block state")
            }
        }
    }

    fn get_block(&self, x: u8, y: u8, z: u8) -> BlockState {
        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;
        self.get_block_by_idx(block_number)
    }

    // fn add_block(&self, x: u8, y: u8, z: u8, state: BlockState){
    //     let id = self.id_to_state.iter().position(|x|x == state);
    //
    //
    //
    //
    //
    //     let new_state_count = &self.storage;
    //     let current_max_count = (1 << self.bits_per_block);
    // }
}


pub enum ChunkColumn {
    LowMemory { data: ChunkData<LowMemoryChunkSection> },
    HighMemory { data: ChunkData<HighMemoryChunkSection> },
}

impl ChunkColumn {
    pub fn modify(&mut self, column: ChunkColumn) {
        if let (ChunkColumn::HighMemory { data: left }, ChunkColumn::HighMemory { data: right }) = (self, column) {
            for (idx, new_section) in IntoIterator::into_iter(right.sections).enumerate() {
                if let Some(section) = new_section {
                    left.sections[idx] = Some(section);
                }
            }
        } else {
            panic!("cannot modify low memory chunks");
        }
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, state: BlockState) {
        let section_idx = (y >> 4) as u8;
        let y_offset = y - (section_idx << 4);

        let section_idx = section_idx as usize;
        match self {
            ChunkColumn::LowMemory { data } => {
                let section = data.sections[section_idx].get_or_insert_default();
                section.set_simple_type(x, y_offset, z, state.simple_type());
            }
            ChunkColumn::HighMemory { data } => {
                let section = data.sections[section_idx].get_or_insert_default();
                section.palette.set_block(x, y_offset, z, state);
            }
        }
    }
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> BlockApprox {
        let section_idx = (y >> 4) as u8;
        let y_offset = y - (section_idx << 4);

        let section_idx = section_idx as usize;

        match self {
            ChunkColumn::LowMemory { data: sections } => {
                let section = &sections.sections[section_idx];
                BlockApprox::Estimate(match section {
                    None => SimpleType::WalkThrough,
                    Some(section) => section.get_simple_type(x, y_offset, z)
                })
            }
            ChunkColumn::HighMemory { data: sections } => {
                let section = &sections.sections[section_idx];
                BlockApprox::Realized(match section {
                    None => BlockState(0),
                    Some(section) => section.palette.get_block(x, y_offset, z)
                })
            }
        }
    }
}
