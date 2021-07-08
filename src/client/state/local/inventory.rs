/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 7/6/21, 11:46 PM
 */

use crate::bootstrap::blocks::BlockData;
use crate::client::physics::tools::{Tool, ToolMat};
use crate::protocol::{InterfaceOut, InvAction};
use crate::storage::block::BlockKind;
use crate::types::{ItemNbt, Slot};

#[derive(Debug)]
pub struct ItemStack {
    pub kind: BlockKind,
    pub count: u8,
    pub damage: u16,
    pub nbt: Option<ItemNbt>,
}

impl From<Slot> for Option<ItemStack> {
    fn from(slot: Slot) -> Self {
        if slot.present() {
            let count = slot.item_count.unwrap();
            let id = slot.block_id;
            let kind = BlockKind(id as u32);
            Some(ItemStack::new(kind, count, slot.item_damage.unwrap(), slot.nbt))
        } else {
            None
        }
    }
}

impl ItemStack {
    pub fn new(kind: BlockKind, count: u8, damage: u16, nbt: Option<ItemNbt>) -> ItemStack {
        Self { kind, count, damage, nbt }
    }
}

#[derive(Debug)]
pub struct PlayerInventory {
    slots: [Option<ItemStack>; 46],
    selected: u8,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        const NONE: Option<ItemStack> = None;
        Self {
            slots: [NONE; 46],
            selected: 0,
        }
    }
}

impl PlayerInventory {
    pub fn hotbar(&self) -> &[Option<ItemStack>] {
        &self.slots[36..45]
    }

    pub fn hotbar_mut(&mut self) -> &mut [Option<ItemStack>] {
        &mut self.slots[36..45]
    }

    /// drop a single item in the hotbar (not multiple because anti cheat does not like this)
    /// returns if dropped all
    pub fn drop_hotbar(&mut self, out: &mut impl InterfaceOut) -> bool{
        let idx = (36..45).filter_map(|idx| {
            self.slots[idx].take()?;
            Some(idx)
        }).next();

        if let Some(idx) = idx {
            out.inventory_action(InvAction::CtrlQ(idx as u16));
            false
        } else {
            true
        }
    }

    pub fn current_tool(&self) -> Tool {
        match &self.hotbar()[self.selected as usize] {
            Some(item) => Tool::from(item),
            None => Tool::default(),
        }
    }

    pub fn change_slot(&mut self, idx: u8, out: &mut impl InterfaceOut) {
        if self.selected != idx {
            self.selected = idx;
            out.change_slot(idx);
        }
    }

    pub fn switch_block(&mut self, out: &mut impl InterfaceOut) {
        self.switch_selector(out, |kind| kind.throw_away_block());
    }

    pub fn switch_bucket(&mut self, out: &mut impl InterfaceOut) {
        self.switch_selector(out, |kind| kind.id() == 325 || kind.id() == 326);
    }

    pub fn switch_tool(&mut self, kind: BlockKind, data: &BlockData, out: &mut impl InterfaceOut) -> Tool {
        let tools = self.hotbar().iter()
            .enumerate()
            .filter_map(|(idx, item_stack)| {
                let item_stack = item_stack.as_ref()?;
                let tool = Tool::from(item_stack);
                Some((idx, tool))
            });

        let min_tool = tools.min_by_key(move |(_, tool)| {
            let wait_time = tool.wait_time(kind, false, false, data);

            // bias towards a hand (so we do not lose durability)
            if tool.material == ToolMat::Hand {
                wait_time
            } else {
                wait_time + 1
            }
        });

        match min_tool {
            None => Tool::default(), // no need to switch slots everything is air
            Some((idx, tool)) => {
                self.change_slot(idx as u8, out);
                tool
            }
        }
    }

    pub fn switch_selector(&mut self, out: &mut impl InterfaceOut, mut block: impl FnMut(BlockKind) -> bool) {
        let block_idx = self.hotbar().iter()
            .enumerate()
            .filter_map(|(idx, item_stack)| {
                let item_stack = item_stack.as_ref()?;
                block(item_stack.kind).then(|| idx)
            })
            .next();

        if let Some(idx) = block_idx {
            self.change_slot(idx as u8, out);
        }
    }

    pub fn remove(&mut self, idx: usize) {
        self.slots[idx] = None;
    }

    pub fn add(&mut self, idx: usize, stack: ItemStack) {
        self.slots[idx] = Some(stack);
    }
}
