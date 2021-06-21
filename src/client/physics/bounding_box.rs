use crate::storage::block::BlockLocation;
use crate::storage::world::WorldBlocks;
use crate::types::Location;

/// 0.6F, 1.95F
#[derive(Copy, Clone)]
pub struct BoundingBox {
    pub from: Location,
    pub to: Location,
}

// impl BoundingBox {
//     fn new(origin: Location, width: f64, height: f64) -> BoundingBox {
//         let half_width = width / 2.0;
//         let from = Location::new(origin.x - half_width, origin.y, origin.z - half_width);
//         let to = Location::new(origin.x + half_width, origin.y + height, origin.z + half_width);
//         BoundingBox {
//             from,
//             to,
//         }
//     }
//
//     fn blocks(&self, world_blocks: &WorldBlocks) {
//         let from: BlockLocation = self.from.into();
//         let to: BlockLocation = self.from.into();
//     }
//
//     pub fn new_blocks_y(&mut self, dy: f64, world_block: &WorldBlocks) {
//         world_block.get
//         if dy > 0.0 {
//             self.to.y += dy;
//         } else {
//             self.from.y += dy;
//         }
//     }
// }
