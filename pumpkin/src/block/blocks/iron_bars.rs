use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

type IronBarsProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::BlockBehaviour;
use crate::world::World;

#[pumpkin_block("minecraft:iron_bars")]
pub struct IronBarsBlock;

#[async_trait]
impl BlockBehaviour for IronBarsBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bars_props = IronBarsProperties::default(args.block);
        bars_props.waterlogged = args.replacing.water_source();

        compute_bars_state(bars_props, args.world, args.block, args.position).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let bars_props = IronBarsProperties::from_state_id(args.state_id, args.block);
        compute_bars_state(bars_props, args.world, args.block, args.position).await
    }
}

pub async fn compute_bars_state(
    mut bars_props: IronBarsProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> u16 {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) = world.get_block_and_state(&other_block_pos).await;

        let connected = other_block == block
            || other_block_state.is_side_solid(direction.opposite())
            || other_block.is_tagged_with_by_tag(&tag::Block::C_GLASS_PANES)
            || other_block.is_tagged_with_by_tag(&tag::Block::MINECRAFT_WALLS);

        match direction {
            BlockDirection::North => bars_props.north = connected,
            BlockDirection::South => bars_props.south = connected,
            BlockDirection::West => bars_props.west = connected,
            BlockDirection::East => bars_props.east = connected,
            _ => {}
        }
    }

    bars_props.to_state_id(block)
}
