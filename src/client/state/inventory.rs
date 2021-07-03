/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::storage::block::{BlockKind};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ItemStack {
    kind: BlockKind,
    count: u32
}

impl ItemStack {
    pub fn new(kind: BlockKind, count: u32) -> ItemStack {
        Self {kind, count}
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

    pub fn remove(&mut self, idx: usize){
        self.slots[idx] = None;
    }

    pub fn add(&mut self, idx: usize, stack: ItemStack){
        self.slots[idx] = Some(stack);
    }
}
