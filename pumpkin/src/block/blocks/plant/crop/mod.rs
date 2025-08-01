use std::sync::Arc;

use pumpkin_data::{
    Block,
    BlockDirection::{East, North, South, West},
    block_properties::{
        BlockProperties, EnumVariants, FarmlandLikeProperties, Integer0To7, WheatLikeProperties,
    },
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
use rand::Rng;

use crate::{block::blocks::plant::PlantBlockBase, world::World};

type CropProperties = WheatLikeProperties;
type FarmlandProperties = FarmlandLikeProperties;

pub mod beetroot;
pub mod carrot;
pub mod gourds;
pub mod potatoes;
pub mod torch_flower;
pub mod wheat;

trait CropBlockBase: PlantBlockBase {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block == &Block::FARMLAND
    }

    fn max_age(&self) -> i32 {
        7
    }

    fn get_age(&self, state: u16, block: &Block) -> i32 {
        let props = CropProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(&self, block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = CropProperties::from_state_id(state, block);
        props.age = Integer0To7::from_index(age as u16);
        props.to_state_id(block)
    }

    async fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let age = self.get_age(state, block);
        if age < self.max_age() {
            let f = get_available_moisture(world, pos, block).await;
            if rand::rng().random_range(0..=(25.0 / f).floor() as i64) == 0 {
                world
                    .set_block_state(
                        pos,
                        self.state_with_age(block, state, age + 1),
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
        }
    }

    //TODO add impl for light level
}

pub async fn get_available_moisture(world: &Arc<World>, pos: &BlockPos, block: &Block) -> f32 {
    let mut moisture = 1.0;
    let down_pos = pos.down();

    for dx in -1..=1 {
        for dz in -1..=1 {
            let mut local_moisture = 0.0;

            let (block, block_state) = world
                .get_block_and_state_id(&down_pos.offset(Vector3 { x: dx, y: 0, z: dz }))
                .await;
            if block == &Block::FARMLAND {
                local_moisture = 1.0;
                let props = FarmlandProperties::from_state_id(block_state, block);
                if props.moisture != Integer0To7::L0 {
                    local_moisture = 3.0;
                }
            }

            if dx != 0 || dz != 0 {
                local_moisture /= 4.0;
            }

            moisture += local_moisture;
        }
    }

    let north = pos.offset(North.to_offset());
    let south = pos.offset(South.to_offset());
    let west = pos.offset(West.to_offset());
    let east = pos.offset(East.to_offset());
    let horizontal = world.get_block(&west).await == block || world.get_block(&east).await == block;
    let vertical = world.get_block(&north).await == block || world.get_block(&south).await == block;
    if (horizontal && vertical)
        || world.get_block(&west.offset(North.to_offset())).await == block
        || world.get_block(&east.offset(North.to_offset())).await == block
        || world.get_block(&east.offset(South.to_offset())).await == block
        || world.get_block(&west.offset(South.to_offset())).await == block
    {
        moisture /= 2.0;
    }

    moisture
}
