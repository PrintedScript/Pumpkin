use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_util::GameMode;

use crate::{
    entity::player::Player,
    item::{ItemBehaviour, ItemMetadata},
};

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

#[async_trait]
impl ItemBehaviour for TridentItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }
}
