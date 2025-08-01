use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use uuid::Uuid;

pub struct SnowBallItem;

impl ItemMetadata for SnowBallItem {
    fn ids() -> Box<[u16]> {
        [Item::SNOWBALL.id].into()
    }
}

const POWER: f32 = 1.5;

#[async_trait]
impl ItemBehaviour for SnowBallItem {
    async fn normal_use(&self, _block: &Item, player: &Player) {
        let position = player.position();
        let world = player.world().await;
        world
            .play_sound(
                Sound::EntitySnowballThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
            )
            .await;
        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            position,
            &EntityType::SNOWBALL,
            false,
        );
        let snowball = ThrownItemEntity::new(entity, &player.living_entity.entity);
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        snowball.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
        world.spawn_entity(Arc::new(snowball)).await;
    }
}
