use interfaces::types::{block_data::BlockData, BlockKind};

use crate::{
    client::physics::tools::{Tool, ToolMat},
    protocol::{InterfaceOut, InvAction},
    types::{ItemNbt, Slot},
};

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
            Some(ItemStack::new(
                kind,
                count,
                slot.item_damage.unwrap(),
                slot.nbt,
            ))
        } else {
            None
        }
    }
}

impl ItemStack {
    pub const fn new(kind: BlockKind, count: u8, damage: u16, nbt: Option<ItemNbt>) -> Self {
        Self {
            kind,
            count,
            damage,
            nbt,
        }
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

    /// drop a single item in the hotbar (not multiple because anti cheat does
    /// not like this) returns if dropped all
    pub fn drop_hotbar(&mut self, out: &mut dyn InterfaceOut) -> bool {
        let idx = (36..45).find_map(|idx| {
            #[allow(clippy::indexing_slicing)]
            self.slots[idx].take()?;
            Some(idx)
        });

        if let Some(idx) = idx {
            out.inventory_action(InvAction::CtrlQ(idx as u16));
            false
        } else {
            true
        }
    }

    #[allow(unused)]
    pub fn current_tool(&self) -> Tool {
        match &self.hotbar()[self.selected as usize] {
            Some(item) => Tool::from(item),
            None => Tool::default(),
        }
    }

    pub fn current(&self) -> Option<&ItemStack> {
        self.hotbar()[self.selected as usize].as_ref()
    }

    pub fn change_slot(&mut self, idx: u8, out: &mut dyn InterfaceOut) {
        if self.selected != idx {
            self.selected = idx;
            out.change_slot(idx);
        }
    }

    pub fn switch_block(&mut self, out: &mut dyn InterfaceOut) {
        self.switch_selector(out, BlockKind::throw_away_block);
    }

    /// true if successful
    pub fn switch_food(&mut self, data: &BlockData, out: &mut dyn InterfaceOut) -> bool {
        self.switch_selector(out, |kind| data.is_food(kind.id()))
    }

    pub fn switch_bucket(&mut self, out: &mut dyn InterfaceOut) {
        self.switch_selector(out, |kind| kind.id() == 325 || kind.id() == 326);
    }

    pub fn switch_tool(
        &mut self,
        kind: BlockKind,
        data: &BlockData,
        out: &mut dyn InterfaceOut,
    ) -> Tool {
        let tools = self.hotbar().iter().enumerate().map(|(idx, item_stack)| {
            let tool = match item_stack.as_ref() {
                Some(stack) => Tool::from(stack),
                None => Tool::default(),
            };
            (idx, tool)
        });

        let (best_idx, best_tool) = tools
            .min_by_key(move |(_, tool)| {
                let wait_time = tool.wait_time(kind, false, false, data);

                // bias towards a hand (so we do not lose durability)
                if tool.material == ToolMat::Hand {
                    wait_time
                } else {
                    wait_time + 1
                }
            })
            .unwrap();

        self.change_slot(best_idx as u8, out);
        best_tool
    }

    pub fn switch_selector(
        &mut self,
        out: &mut dyn InterfaceOut,
        mut block: impl FnMut(BlockKind) -> bool,
    ) -> bool {
        let block_idx = self
            .hotbar()
            .iter()
            .enumerate()
            .find_map(|(idx, item_stack)| {
                let item_stack = item_stack.as_ref()?;
                block(item_stack.kind).then_some(idx)
            });

        if let Some(idx) = block_idx {
            self.change_slot(idx as u8, out);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, idx: usize) {
        self.slots[idx] = None;
    }

    pub fn add(&mut self, idx: usize, stack: ItemStack) {
        self.slots[idx] = Some(stack);
    }
}
