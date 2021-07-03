/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::storage::block::{BlockKind};
use crate::types::{Nbt, Slot};
use crate::protocol::{InterfaceOut, InvAction};

#[derive(Debug)]
pub struct ItemStack {
    pub kind: BlockKind,
    pub count: u8,
    pub damage: u16,
    pub nbt: Option<Nbt>
}


impl ItemStack {
    pub fn new(kind: BlockKind, count: u8, damage: u16, nbt: Option<Nbt>) -> ItemStack {
        Self {kind, count, damage, nbt}
    }
}

#[derive(Debug)]
pub struct PlayerInventory {
    slots: [Option<ItemStack>; 46]
}

impl Default for PlayerInventory {
    fn default() -> Self {
        const NONE: Option<ItemStack> = None;
        Self {
            slots: [NONE; 46]
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

    pub fn drop_hotbar(&mut self, out: &mut impl InterfaceOut){
        for idx in 36..45 {
            if self.slots[idx].take().is_some() {
                out.inventory_action(InvAction::Q(idx as u16))
            }
        }
    }

    pub fn remove(&mut self, idx: usize){
        self.slots[idx] = None;
    }

    pub fn add(&mut self, idx: usize, stack: ItemStack){
        self.slots[idx] = Some(stack);
    }
}
