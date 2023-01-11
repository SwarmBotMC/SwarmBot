pub use interfaces::types::*;
use serde::{Deserialize, Serialize};
use swarm_bot_packets::{
    read::{ByteReadable, ByteReader},
    write::{ByteWritable, ByteWriter},
};

use crate::client::state::local::inventory::ItemStack;

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemNbt {
    pub ench: Option<Vec<Enchantment>>,
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

/// <https://wiki.vg/Slot_Data>
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
    pub const EMPTY: Self = {
        Self {
            block_id: -1,
            item_count: None,
            item_damage: None,
            nbt: None,
        }
    };

    pub const fn present(&self) -> bool {
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
                Some(nbt) => writer.write(nbt),
            };
        }
    }
}

impl ByteReadable for Slot {
    fn read_from_bytes(byte_reader: &mut ByteReader) -> Self {
        let block_id: i16 = byte_reader.read();

        if block_id == -1 {
            Self {
                block_id,
                item_count: None,
                item_damage: None,
                nbt: None,
            }
        } else {
            let item_count = byte_reader.read();
            let item_damage = byte_reader.read();

            let first: u8 = byte_reader.read();
            let nbt = (first != 0).then(|| {
                byte_reader.back(1);
                byte_reader.read()
            });

            Self {
                block_id,
                item_count: Some(item_count),
                item_damage: Some(item_damage),
                nbt,
            }
        }
    }
}
