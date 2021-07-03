/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/29/21, 8:41 PM
 */

use crate::storage::block::{BlockKind};
use crate::types::{Nbt};
use crate::protocol::{InterfaceOut, InvAction};
use itertools::Itertools;
use crate::client::physics::tools::Tool;
use float_ord::FloatOrd;

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
    slots: [Option<ItemStack>; 46],
    selected: u8,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        const NONE: Option<ItemStack> = None;
        Self {
            slots: [NONE; 46],
            selected: 0
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

    pub fn current_tool(&self) -> Tool {
        match &self.hotbar()[self.selected as usize] {
            Some(item) => item.kind.to_tool(),
            None => Tool::default(),
        }
    }

    pub fn change_slot(&mut self, idx: u8, out: &mut impl InterfaceOut){
        self.selected = idx;
        out.change_slot(idx);
    }

    pub fn switch_tool(&mut self, out: &mut impl InterfaceOut) -> Tool{
        let tools = self.hotbar().iter()
            .enumerate()
            .filter_map(|(idx, item_stack)| {
                let item_stack = item_stack.as_ref()?;
                let tool = item_stack.kind.to_tool();
                Some((idx, tool))
            });

        match tools.max_by_key(move |(_, tool)| FloatOrd(tool.material.strength())) {
            None => Tool::default(),
            Some((idx, tool)) => {
                self.change_slot(idx as u8, out);
                tool
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
