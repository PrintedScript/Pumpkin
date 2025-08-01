use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BedPart;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::bed::BedBlockEntity;
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, NormalUseArgs, OnPlaceArgs, PlacedArgs,
};
use crate::entity::{Entity, EntityBase};
use crate::world::World;

type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;

const NO_SLEEP_IDS: &[u16] = &[
    EntityType::BLAZE.id,
    EntityType::BOGGED.id,
    EntityType::SKELETON.id,
    EntityType::STRAY.id,
    EntityType::WITHER_SKELETON.id,
    EntityType::BREEZE.id,
    EntityType::CREAKING.id,
    EntityType::CREEPER.id,
    EntityType::DROWNED.id,
    EntityType::ENDERMITE.id,
    EntityType::EVOKER.id,
    EntityType::GIANT.id,
    EntityType::GUARDIAN.id,
    EntityType::ELDER_GUARDIAN.id,
    EntityType::ILLUSIONER.id,
    EntityType::OCELOT.id,
    EntityType::PIGLIN.id,
    EntityType::PIGLIN_BRUTE.id,
    EntityType::PILLAGER.id,
    EntityType::PHANTOM.id,
    EntityType::RAVAGER.id,
    EntityType::SILVERFISH.id,
    EntityType::SPIDER.id,
    EntityType::CAVE_SPIDER.id,
    EntityType::VEX.id,
    EntityType::VINDICATOR.id,
    EntityType::WARDEN.id,
    EntityType::WITCH.id,
    EntityType::WITHER.id,
    EntityType::ZOGLIN.id,
    EntityType::ZOMBIE.id,
    EntityType::ZOMBIE_VILLAGER.id,
    EntityType::HUSK.id,
    EntityType::ENDERMAN.id,
    EntityType::ZOMBIFIED_PIGLIN.id,
];

#[pumpkin_block_from_tag("minecraft:beds")]
pub struct BedBlock;

#[async_trait]
impl BlockBehaviour for BedBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player {
            let facing = player.living_entity.entity.get_horizontal_facing();
            return args
                .block_accessor
                .get_block_state(args.position)
                .await
                .replaceable()
                && args
                    .block_accessor
                    .get_block_state(&args.position.offset(facing.to_offset()))
                    .await
                    .replaceable();
        }
        false
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bed_props = BedProperties::default(args.block);

        bed_props.facing = args.player.living_entity.entity.get_horizontal_facing();
        bed_props.part = BedPart::Foot;

        bed_props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let bed_entity = BedBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(bed_entity)).await;

        let mut bed_head_props = BedProperties::default(args.block);
        bed_head_props.facing = BedProperties::from_state_id(args.state_id, args.block).facing;
        bed_head_props.part = BedPart::Head;

        let bed_head_pos = args.position.offset(bed_head_props.facing.to_offset());
        args.world
            .set_block_state(
                &bed_head_pos,
                bed_head_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            )
            .await;

        let bed_head_entity = BedBlockEntity::new(bed_head_pos);
        args.world.add_block_entity(Arc::new(bed_head_entity)).await;
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        let bed_props = BedProperties::from_state_id(args.state.id, args.block);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };

        args.world
            .break_block(
                &other_half_pos,
                Some(args.player.clone()),
                if args.player.gamemode.load() == GameMode::Creative {
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_NEIGHBORS
                } else {
                    BlockFlags::NOTIFY_NEIGHBORS
                },
            )
            .await;
    }

    #[allow(clippy::too_many_lines)]
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position).await;
        let bed_props = BedProperties::from_state_id(state_id, args.block);

        let (bed_head_pos, bed_foot_pos) = if bed_props.part == BedPart::Head {
            (
                *args.position,
                args.position
                    .offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (
                args.position.offset(bed_props.facing.to_offset()),
                *args.position,
            )
        };

        // Explode if not in the overworld
        if args.world.dimension_type != VanillaDimensionType::Overworld {
            args.world
                .break_block(&bed_head_pos, None, BlockFlags::SKIP_DROPS)
                .await;
            args.world
                .break_block(&bed_foot_pos, None, BlockFlags::SKIP_DROPS)
                .await;

            args.world
                .explode(args.server, bed_head_pos.to_centered_f64(), 5.0)
                .await;

            return BlockActionResult::SuccessServer;
        }

        // Make sure the bed is not obstructed
        if args
            .world
            .get_block_state(&bed_head_pos.up())
            .await
            .is_solid()
            || args
                .world
                .get_block_state(&bed_head_pos.up())
                .await
                .is_solid()
        {
            args.player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.obstructed", []),
                    true,
                )
                .await;
            return BlockActionResult::SuccessServer;
        }

        // Make sure the bed is not occupied
        if bed_props.occupied {
            // TODO: Wake up villager

            args.player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.occupied", []),
                    true,
                )
                .await;
            return BlockActionResult::SuccessServer;
        }

        // Make sure player is close enough
        if !args
            .player
            .position()
            .is_within_bounds(bed_head_pos.to_f64(), 3.0, 3.0, 3.0)
            && !args
                .player
                .position()
                .is_within_bounds(bed_foot_pos.to_f64(), 3.0, 3.0, 3.0)
        {
            args.player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.too_far_away", []),
                    true,
                )
                .await;
            return BlockActionResult::SuccessServer;
        }

        // Set respawn point
        if args
            .player
            .set_respawn_point(
                args.world.dimension_type,
                bed_head_pos,
                args.player.get_entity().yaw.load(),
            )
            .await
        {
            args.player
                .send_system_message(&TextComponent::translate("block.minecraft.set_spawn", []))
                .await;
        }

        // Make sure the time and weather allows sleep
        if !can_sleep(args.world).await {
            args.player
                .send_system_message_raw(
                    &TextComponent::translate("block.minecraft.bed.no_sleep", []),
                    true,
                )
                .await;
            return BlockActionResult::SuccessServer;
        }

        // Make sure there are no monsters nearby
        for entity in args.world.entities.read().await.values() {
            if !entity_prevents_sleep(entity.get_entity()) {
                continue;
            }

            let pos = entity.get_entity().pos.load();
            if pos.is_within_bounds(bed_head_pos.to_f64(), 8.0, 5.0, 8.0)
                || pos.is_within_bounds(bed_foot_pos.to_f64(), 8.0, 5.0, 8.0)
            {
                args.player
                    .send_system_message_raw(
                        &TextComponent::translate("block.minecraft.bed.not_safe", []),
                        true,
                    )
                    .await;
                return BlockActionResult::SuccessServer;
            }
        }

        args.player.sleep(bed_head_pos).await;
        Self::set_occupied(true, args.world, args.block, args.position, state_id).await;

        BlockActionResult::SuccessServer
    }
}

impl BedBlock {
    pub async fn set_occupied(
        occupied: bool,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        state_id: u16,
    ) {
        let mut bed_props = BedProperties::from_state_id(state_id, block);
        bed_props.occupied = occupied;
        world
            .set_block_state(
                block_pos,
                bed_props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;

        let other_half_pos = if bed_props.part == BedPart::Head {
            block_pos.offset(bed_props.facing.opposite().to_offset())
        } else {
            block_pos.offset(bed_props.facing.to_offset())
        };
        bed_props.part = if bed_props.part == BedPart::Head {
            BedPart::Foot
        } else {
            BedPart::Head
        };
        world
            .set_block_state(
                &other_half_pos,
                bed_props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }
}

async fn can_sleep(world: &Arc<World>) -> bool {
    let time = world.level_time.lock().await;
    let weather = world.weather.lock().await;

    if weather.thundering {
        true
    } else if weather.raining {
        time.time_of_day > 12010 && time.time_of_day < 23991
    } else {
        time.time_of_day > 12542 && time.time_of_day < 23459
    }
}

fn entity_prevents_sleep(entity: &Entity) -> bool {
    NO_SLEEP_IDS.contains(&entity.entity_type.id)
}
