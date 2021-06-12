use crate::storage::block::{BlockApprox, BlockState, SimpleType};

const SECTION_ELEMENTS: usize = 16 * 16 * 16;
const BITS_PER_ENUM: usize = 2;
const SECTION_BYTES: usize = SECTION_ELEMENTS * BITS_PER_ENUM / 8;

#[repr(packed)]
pub struct HighMemoryChunkSection {
    pub palette: Palette,
}

#[repr(packed)]
pub struct LowMemoryChunkSection {
    storage: [u8; SECTION_BYTES],
}

impl LowMemoryChunkSection {
    fn get_simple_type(&self, x: u8, y: u8, z: u8) -> SimpleType {
        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;

        // 2 bits per block
        let idx = block_number >> 2;
        let offset = block_number - (idx << 2);

        let mut res = self.storage[idx];
        res = res >> offset;
        res &= 0b11;

        SimpleType::from(res)
    }

    fn set_simple_type(&self, x: u8, y: u8, z: u8, input: SimpleType) {
        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;

        // 2 bits per block
        let idx = block_number >> 2;
        let offset = block_number - (idx << 2);

        let mut block = self.storage[idx];

        let id = input.id();

        let zero_out = !(0b11 << offset);
        block &= zero_out;

        block |= id << offset;
    }
}

pub struct ChunkData<T> {
    pub section: [Option<T>; 16],
}

const SECTION_HEIGHT: usize = 16;
const SECTION_WIDTH: usize = 16;


#[repr(packed)]
pub struct Palette {
    bits_per_block: u8,
    id_to_state: Option<Vec<BlockState>>,
    storage: Vec<u64>,
}

impl Palette {

    // fn set_block_compressed_id(&self, x: u8, y: u8, z: u8, value: u32) {
    //
    //     let bits_per_block = self.bits_per_block;
    //
    //     let indv_value_mask = (1 << bits_per_block) - 1;
    //
    //     let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;
    //     let start_long = (block_number * bits_per_block) / 64;
    //     let start_offset = (block_number * bits_per_block) % 64;
    //     let end_long = ((block_number + 1) * bits_per_block - 1) / 64;
    //
    //     let value = value & indv_value_mask;
    //
    //     data[start_long] |= (value << start_offset);
    //
    //     if start_long != end_long {
    //         data[end_long] = (value >> (64 - start_offset));
    //     }
    // }

    fn get_block(&self, x: u8, y: u8, z: u8) -> BlockState {
        let data_arr = &self.storage;

        let bits_per_block = self.bits_per_block as usize;

        let indv_value_mask = (1 << bits_per_block) - 1;

        let block_number = (((y as usize * SECTION_HEIGHT) + z as usize) * SECTION_WIDTH) + x as usize;
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
                let value = map.get(data as usize).expect("internal chunk error getting block state");
                value.clone()
            }
        }
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


pub enum Chunk {
    LowMemory { sections: ChunkData<LowMemoryChunkSection> },
    HighMemory { sections: ChunkData<HighMemoryChunkSection> },
}


impl Chunk {
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> BlockApprox {
        let section_idx = (y >> 4) as u8;
        let y_offset = y - (section_idx << 4);

        let section_idx = section_idx as usize;

        match self {
            Chunk::LowMemory { sections } => {
                let section = &sections.section[section_idx];
                BlockApprox::Estimate(match section {
                    None => SimpleType::WalkThrough,
                    Some(section) => section.get_simple_type(x, y_offset, z)
                })
            }
            Chunk::HighMemory { sections } => {
                let section = &sections.section[section_idx];
                BlockApprox::Realized(match section {
                    None => BlockState(0),
                    Some(section) => section.palette.get_block(x, y_offset, z)
                })
            }
        }
    }
}
