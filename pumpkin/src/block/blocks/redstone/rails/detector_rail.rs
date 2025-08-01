use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::CanPlaceAtArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::PlacedArgs;

use super::RailProperties;
use super::common::{
    can_place_rail_at, compute_placed_rail_shape, rail_placement_is_valid,
    update_flanking_rails_shape,
};

#[pumpkin_block("minecraft:detector_rail")]
pub struct DetectorRailBlock;

#[async_trait]
impl BlockBehaviour for DetectorRailBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut rail_props = RailProperties::default(args.block);
        let player_facing = args.player.living_entity.entity.get_horizontal_facing();

        rail_props.set_waterlogged(args.replacing.water_source());
        rail_props.set_straight_shape(
            compute_placed_rail_shape(args.world, args.position, player_facing).await,
        );

        rail_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        update_flanking_rails_shape(args.world, args.block, args.state_id, args.position).await;
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !rail_placement_is_valid(args.world, args.block, args.position).await {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position).await
    }
}
