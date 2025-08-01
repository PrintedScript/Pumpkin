use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::entity::player::Player;
use crate::item::items::ignite::ignition::Ignition;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;

pub struct FireChargeItem;

impl ItemMetadata for FireChargeItem {
    fn ids() -> Box<[u16]> {
        [Item::FIRE_CHARGE.id].into()
    }
}

#[async_trait]
impl ItemBehaviour for FireChargeItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _block: &Block,
        _server: &Server,
    ) {
        Ignition::ignite_block(
            |world: Arc<World>, pos: BlockPos, new_state_id: u16| async move {
                world
                    .set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;

                world
                    .play_block_sound(Sound::ItemFirechargeUse, SoundCategory::Blocks, pos)
                    .await;
            },
            _item,
            player,
            location,
            face,
            _block,
            _server,
        )
        .await;
    }
}
