use std::sync::Arc;

use crate::{
    block::{
        registry::BlockActionResult,
        {
            BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnScheduledTickArgs,
            UseWithItemArgs,
        },
    },
    entity::{Entity, item::ItemEntity},
    world::World,
};
use async_trait::async_trait;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, ComposterLikeProperties, EnumVariants, Integer0To8},
    composter_increase_chance::get_composter_increase_chance_from_item_id,
    entity::EntityType,
    item::Item,
    world::WorldEvent,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, item::ItemStack, tick::TickPriority, world::BlockFlags};
use rand::Rng;
use uuid::Uuid;

#[pumpkin_block("minecraft:composter")]
pub struct ComposterBlock;

#[async_trait]
impl BlockBehaviour for ComposterBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position).await;
        let props = ComposterLikeProperties::from_state_id(state_id, args.block);
        if props.get_level() == 8 {
            self.clear_composter(args.world, args.position, state_id, args.block)
                .await;
        }

        BlockActionResult::Pass
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position).await;
        let props = ComposterLikeProperties::from_state_id(state_id, args.block);
        let level = props.get_level();
        if level == 8 {
            self.clear_composter(args.world, args.position, state_id, args.block)
                .await;
        }
        if level < 7 {
            if let Some(chance) =
                get_composter_increase_chance_from_item_id(args.item_stack.lock().await.item.id)
            {
                if level == 0 || rand::rng().random_bool(f64::from(chance)) {
                    self.update_level_composter(
                        args.world,
                        args.position,
                        state_id,
                        args.block,
                        level + 1,
                    )
                    .await;
                    args.world
                        .sync_world_event(WorldEvent::ComposterUsed, *args.position, 1)
                        .await;
                }
            }
        }
        BlockActionResult::Consume
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position).await;
        let props = ComposterLikeProperties::from_state_id(state_id, args.block);
        let level = props.get_level();
        if level == 7 {
            self.update_level_composter(args.world, args.position, state_id, args.block, level + 1)
                .await;
        }
    }

    async fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = ComposterLikeProperties::from_state_id(args.state.id, args.block);
        Some(props.get_level())
    }
}

impl ComposterBlock {
    pub async fn update_level_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
        level: u8,
    ) {
        let mut props = ComposterLikeProperties::from_state_id(state_id, block);
        props.set_level(level);
        world
            .set_block_state(location, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        if level == 7 {
            world
                .schedule_block_tick(block, *location, 20, TickPriority::Normal)
                .await;
        }
    }

    pub async fn clear_composter(
        &self,
        world: &Arc<World>,
        location: &BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) {
        self.update_level_composter(world, location, state_id, block, 0)
            .await;

        let item_position = {
            let mut rng = rand::rng();
            location.to_centered_f64().add_raw(
                rng.random_range(-0.35..=0.35),
                rng.random_range(-0.35..=0.35) + 0.51,
                rng.random_range(-0.35..=0.35),
            )
        };

        let item_entity = ItemEntity::new(
            Entity::new(
                Uuid::new_v4(),
                world.clone(),
                item_position,
                &EntityType::ITEM,
                false,
            ),
            ItemStack::new(1, &Item::BONE_MEAL),
        )
        .await;

        world.spawn_entity(Arc::new(item_entity)).await;
    }
}

pub trait ComposterPropertiesEx {
    fn get_level(&self) -> u8;
    fn set_level(&mut self, level: u8);
}

impl ComposterPropertiesEx for ComposterLikeProperties {
    fn get_level(&self) -> u8 {
        self.level.to_index() as u8
    }
    fn set_level(&mut self, level: u8) {
        self.level = Integer0To8::from_index(u16::from(level));
    }
}
