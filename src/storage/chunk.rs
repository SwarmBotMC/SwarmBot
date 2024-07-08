#![allow(clippy::unwrap_used, clippy::cast_possible_wrap)]

use std::collections::HashMap;

use interfaces::types::{BlockApprox, BlockLocation, BlockState, ChunkLocation, SimpleType};

use crate::default;

const SECTION_ELEMENTS: usize = 16 * 16 * 16;
const BITS_PER_ENUM: usize = 2;
const SECTION_BYTES: usize = SECTION_ELEMENTS * BITS_PER_ENUM / 8;

#[derive(Default)]
pub struct HighMemoryChunkSection {
    pub palette: Palette,
}

impl HighMemoryChunkSection {
    pub const fn new(palette: Palette) -> Self {
        Self { palette }
    }
}

#[repr(packed)]
pub struct LowMemoryChunkSection {
    storage: [u8; SECTION_BYTES],
}

impl Default for LowMemoryChunkSection {
    fn default() -> Self {
        Self {
            storage: [0; SECTION_BYTES],
        }
    }
}

pub const fn bits_needed(number: usize) -> u8 {
    // 1 bit can encode 2
    let mut start = 2;
    let mut bits_needed = 1;

    while start < number {
        start <<= 1;
        bits_needed += 1;
    }

    bits_needed
}

impl LowMemoryChunkSection {
    #[allow(clippy::indexing_slicing)]
    fn get_simple_type(&self, x: u8, y: u8, z: u8) -> SimpleType {
        let block_number =
            (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;

        // 2 bits per block
        let idx = block_number >> 2;
        let offset = block_number - (idx << 2);

        let mut res = self.storage[idx];
        res >>= offset;
        res &= 0b11;

        SimpleType::from(res)
    }

    #[allow(unused, clippy::indexing_slicing)]
    fn set_simple_type(&mut self, x: u8, y: u8, z: u8, input: SimpleType) {
        let block_number =
            (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;

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

#[derive(Default)]
pub struct ChunkData<T> {
    pub sections: [Option<Box<T>>; 16],
}

impl<T> ChunkData<T> {
    #[allow(unused)]
    pub fn block_location(location: ChunkLocation, idx: usize) -> BlockLocation {
        let base_x = location.0 << 4;
        let base_z = location.1 << 4;

        let x = idx % 16;
        let leftover = idx >> 4;
        let z = leftover % 16;
        let y = leftover / 16;
        BlockLocation::new(base_x + x as i32, y as i16, base_z + z as i32)
    }

    #[allow(unused)]
    fn highest_mut(&mut self) -> Option<&mut T> {
        self.sections
            .iter_mut()
            .rev()
            .flatten()
            .next()
            .map(std::convert::AsMut::as_mut)
    }

    #[allow(unused)]
    fn lowest_mut(&mut self) -> Option<&mut T> {
        self.sections
            .iter_mut()
            .flatten()
            .next()
            .map(std::convert::AsMut::as_mut)
    }
}

impl ChunkData<HighMemoryChunkSection> {
    #[allow(clippy::indexing_slicing)]
    pub fn all_at(&self, y: u8) -> [BlockState; 256] {
        let section_idx = y >> 4;

        let chunk_y = y - (section_idx << 4);

        let mut res = [BlockState::AIR; 16 * 16];
        let Some(section) = self.sections[section_idx as usize].as_ref() else {
            return res;
        };

        let mut idx = 0;

        for z in 0..16 {
            for x in 0..16 {
                let state = section.palette.get_block(x, chunk_y, z);
                res[idx] = state;
                idx += 1;
            }
        }

        res
    }
    pub fn select_up<'a>(
        &'a self,
        mut selector: impl FnMut(BlockState) -> bool + 'a,
    ) -> impl Iterator<Item = usize> + 'a {
        self.sections
            .iter()
            .enumerate()
            .filter_map(|(chunk_idx, section)| section.as_ref().map(|sec| (chunk_idx << 12, sec)))
            .flat_map(|(idx_start, section)| {
                IntoIterator::into_iter(section.palette.all_states())
                    .enumerate()
                    .map(move |(idx, state)| (idx_start + idx, state))
            })
            .filter(move |(_, state)| selector(*state))
            .map(|(idx, _)| idx)
    }

    #[allow(unused)]
    pub fn select_locs<'a>(
        &'a self,
        location: ChunkLocation,
        selector: impl FnMut(BlockState) -> bool + 'a,
    ) -> impl Iterator<Item = BlockLocation> + 'a {
        self.select_up(selector)
            .map(move |idx| Self::block_location(location, idx))
    }

    // TODO: remove duplicate code... is it even possible?
    #[allow(unused)]
    pub fn select_down<'a>(
        &'a self,
        mut selector: impl FnMut(BlockState) -> bool + 'a,
    ) -> impl Iterator<Item = usize> + 'a {
        self.sections
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(chunk_idx, section)| section.as_ref().map(|sec| (chunk_idx << 12, sec)))
            .flat_map(|(idx_start, section)| {
                IntoIterator::into_iter(section.palette.all_states())
                    .enumerate()
                    .rev()
                    .map(move |(idx, state)| (idx_start + idx, state))
            })
            .filter(move |(_, state)| selector(*state))
            .map(|(idx, _)| idx)
    }
}

const SECTION_HEIGHT: usize = 16;
const SECTION_WIDTH: usize = 16;

pub struct Palette {
    bits_per_block: u8,
    id_to_state: Option<Vec<BlockState>>,

    /// invariant: must always be at bits_per_block * 4096 / 64 ... if 4 we have
    /// 256 ... if we have 1 we get 64
    storage: Vec<u64>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            // the smallest bpb
            bits_per_block: 1,
            id_to_state: Some(vec![BlockState::AIR]),
            storage: vec![0; 64],
        }
    }
}

impl Palette {
    pub const fn direct(storage: Vec<u64>) -> Self {
        Self {
            bits_per_block: 13,
            id_to_state: None,
            storage,
        }
    }

    pub fn indirect(bits_per_block: u8, id_to_state: Vec<BlockState>, storage: Vec<u64>) -> Self {
        assert!(bits_per_block >= 4);
        assert!(bits_per_block <= 8);
        assert_eq!(storage.len(), 4096 / 64 * bits_per_block as usize);
        Self {
            bits_per_block,
            id_to_state: Some(id_to_state),
            storage,
        }
    }

    #[allow(unused, clippy::indexing_slicing)]
    pub fn all_states(&self) -> [BlockState; 4096] {
        let mut res = [BlockState::AIR; 4096];
        (0..4096).for_each(|i| res[i] = self.get_block_by_idx(i));
        res
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, state: BlockState) {
        let value = match self.id_to_state.as_mut() {
            None => state.0,
            Some(id_to_state) => {
                // we only have to modify the map if we do not have the state
                let value = id_to_state.iter().position(|&r| r == state);

                match value {
                    None => {
                        let new_len = id_to_state.len() + 1;
                        let required_bits = bits_needed(new_len);

                        id_to_state.push(state);

                        if required_bits > self.bits_per_block {
                            let (required_bits, reverse_map) = if required_bits <= 8 {
                                let reverse_map: HashMap<_, _> = id_to_state
                                    .iter()
                                    .enumerate()
                                    .map(|(k, v)| (*v, k))
                                    .collect();

                                (required_bits.max(4), Some(reverse_map))
                            } else {
                                self.id_to_state = None;

                                (13, None)
                            };

                            // debug_println!("expand bits {} -> {} ... reverse_map {:?}",
                            // self.bits_per_block, required_bits, reverse_map);

                            // we have to recreate the palette

                            // TODO: we could modify states with new block id and instantly return
                            let states = self.all_states();

                            // update bits per block
                            self.bits_per_block = required_bits;

                            let required_bits = required_bits as usize;

                            let indv_value_mask = (1 << required_bits) - 1;

                            assert!(required_bits >= 4);
                            let new_data_size = 4096 * required_bits / 64;
                            let mut storage = vec![0_u64; new_data_size];

                            for (block_number, state) in IntoIterator::into_iter(states).enumerate()
                            {
                                let start_long = (block_number * required_bits) / 64;
                                let start_offset = (block_number * required_bits) % 64;
                                let end_long = ((block_number + 1) * required_bits - 1) / 64;

                                let value = match reverse_map.as_ref() {
                                    None => u64::from(state.0),
                                    Some(reverse_map) => *reverse_map.get(&state).unwrap() as u64,
                                };

                                let value = value & indv_value_mask;

                                storage[start_long] |= value << start_offset;

                                if start_long != end_long {
                                    storage[end_long] = value >> (64 - start_offset);
                                }
                            }

                            self.storage = storage;
                        }
                        (new_len - 1) as u32
                    }
                    Some(value) => value as u32,
                }
            }
        };

        let value = u64::from(value);
        let indv_value_mask = (1 << self.bits_per_block) - 1;

        let block_number =
            (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;
        let bits_per_block = self.bits_per_block as usize;
        let start_long = (block_number * bits_per_block) / 64;
        let start_offset = (block_number * bits_per_block) % 64;
        let end_long = ((block_number + 1) * bits_per_block - 1) / 64;

        // zero out TODO: thread 'main' panicked at 'index out of bounds: the len is 0
        // but the index is 0', below...
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
            Some(map) => *map
                .get(data as usize)
                .expect("internal chunk error getting block state"),
        }
    }

    fn get_block(&self, x: u8, y: u8, z: u8) -> BlockState {
        let block_number =
            (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;
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

/// A chunk storage module
pub enum Column {
    /// low memory data. each block takes 2 bits
    #[allow(unused)]
    LowMemory {
        /// the data
        data: ChunkData<LowMemoryChunkSection>,
    },
    /// high memory (yet still compressed data)
    HighMemory {
        /// the data
        data: ChunkData<HighMemoryChunkSection>,
    },
}

impl Default for Column {
    fn default() -> Self {
        Self::HighMemory {
            data: ChunkData::default(),
        }
    }
}

impl Column {
    /// modify a column
    /// TODO: is this needed? can we just use *
    pub fn modify(&mut self, column: Self) {
        if let (Self::HighMemory { data: left }, Self::HighMemory { data: right }) = (self, column)
        {
            for (idx, new_section) in IntoIterator::into_iter(right.sections).enumerate() {
                if let Some(section) = new_section {
                    left.sections[idx] = Some(section);
                }
            }
        } else {
            panic!("cannot modify low memory chunks");
        }
    }

    /// set a block in the column
    pub fn set_block(&mut self, x: u8, y: u8, z: u8, state: BlockState) {
        let section_idx = y >> 4;
        let y_offset = y - (section_idx << 4);

        let section_idx = section_idx as usize;
        match self {
            Self::LowMemory { data } => {
                let section = data.sections[section_idx].get_or_insert_with(default);
                section.set_simple_type(x, y_offset, z, state.simple_type());
            }
            Self::HighMemory { data } => {
                let section = data.sections[section_idx].get_or_insert_with(default);
                section.palette.set_block(x, y_offset, z, state);
            }
        }
    }

    /// get a block in the column
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> BlockApprox {
        let section_idx = y >> 4;
        let y_offset = y - (section_idx << 4);

        let section_idx = section_idx as usize;

        match self {
            Self::LowMemory { data: sections } => {
                let section = &sections.sections[section_idx];
                BlockApprox::Estimate(match section {
                    None => SimpleType::WalkThrough,
                    Some(section) => section.get_simple_type(x, y_offset, z),
                })
            }
            Self::HighMemory { data: sections } => {
                let section = &sections.sections[section_idx];
                BlockApprox::Realized(match section {
                    None => BlockState(0),
                    Some(section) => section.palette.get_block(x, y_offset, z),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use interfaces::types::BlockState;
    use itertools::Itertools;

    use crate::storage::chunk::{bits_needed, Palette};

    #[test]
    fn test_bits_needed() {
        assert_eq!(1, bits_needed(1)); // 000
        assert_eq!(1, bits_needed(2)); // 001
        assert_eq!(2, bits_needed(3)); // 010
        assert_eq!(2, bits_needed(4)); // 011
        assert_eq!(3, bits_needed(5)); // 100
    }

    #[test]
    fn test_palette_expand() {
        let mut palette = Palette::default();

        // test empty palette is all iar
        for ((x, y), z) in (0..16).cartesian_product(0..16).cartesian_product(0..16) {
            assert_eq!(palette.get_block(x, y, z), BlockState::AIR);
        }

        // test adding stone is correct
        for ((x, y), z) in (0..16).cartesian_product(0..16).cartesian_product(0..16) {
            let sum = x + y + z;
            if sum % 2 == 0 {
                palette.set_block(x, y, z, BlockState(9));
            } else if sum % 3 == 0 {
                palette.set_block(x, y, z, BlockState(37));
            }
        }
        for ((x, y), z) in (0..16).cartesian_product(0..16).cartesian_product(0..16) {
            let sum = x + y + z;
            let block = palette.get_block(x, y, z);
            if sum % 2 == 0 {
                assert_eq!(block, BlockState(9));
            } else if sum % 3 == 0 {
                assert_eq!(block, BlockState(37));
            } else {
                assert_eq!(block, BlockState(0));
            }
        }

        let mut index = 0;
        let mut map = HashMap::new();

        // test adding other blocks is correct
        for ((x, y), z) in (0..16).cartesian_product(0..16).cartesian_product(0..16) {
            let sum = x + y + z;

            let block_state = if primes::is_prime(sum) {
                let idx = map.entry(sum).or_insert_with(|| {
                    index += 1;
                    index
                });

                BlockState(*idx)
            } else {
                BlockState::AIR
            };
            palette.set_block(x as u8, y as u8, z as u8, block_state);
        }

        // test adding other blocks is correct
        for ((x, y), z) in (0..16).cartesian_product(0..16).cartesian_product(0..16) {
            let sum = x + y + z;

            let block_state = if primes::is_prime(sum) {
                let idx = map[&sum];
                BlockState(idx)
            } else {
                BlockState::AIR
            };

            assert_eq!(
                palette.get_block(x as u8, y as u8, z as u8),
                block_state,
                "not eq at {x} {y} {z}"
            );
        }
    }
}
