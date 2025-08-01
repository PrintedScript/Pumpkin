use std::sync::LazyLock;

use banner_pattern::BannerPattern;
use biome::Biome;
use cat::CatVariant;
use chat_type::{ChatType, Decoration};
use chicken::ChickenVariant;
use cow::CowVariant;
use damage_type::DamageType;
use dimension::Dimension;
use enchantment::Enchantment;
use frog::FrogVariant;
use indexmap::IndexMap;
use instrument::Instrument;
use jukebox_song::JukeboxSong;
use paint::Painting;
use pig::PigVariant;
use pumpkin_protocol::java::client::config::RegistryEntry;
use pumpkin_util::{read_data_from_file, resource_location::ResourceLocation};
use serde::{Deserialize, Serialize};
use trim_material::TrimMaterial;
use trim_pattern::TrimPattern;
use wolf::{WolfSoundVariant, WolfVariant};

mod banner_pattern;
mod biome;
mod cat;
mod chat_type;
mod chicken;
mod cow;
mod damage_type;
mod dimension;
mod enchantment;
mod frog;
mod instrument;
mod jukebox_song;
mod paint;
mod pig;
mod trim_material;
mod trim_pattern;
mod wolf;

pub static SYNCED_REGISTRIES: LazyLock<SyncedRegistry> =
    LazyLock::new(|| read_data_from_file!("../../assets/synced_registries.json"));

pub struct Registry {
    pub registry_id: ResourceLocation,
    pub registry_entries: Vec<RegistryEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedRegistry {
    #[serde(rename = "worldgen/biome")]
    biome: IndexMap<String, Biome>,
    chat_type: IndexMap<String, ChatType>,
    trim_pattern: IndexMap<String, TrimPattern>,
    trim_material: IndexMap<String, TrimMaterial>,
    wolf_variant: IndexMap<String, WolfVariant>,
    painting_variant: IndexMap<String, Painting>,
    dimension_type: IndexMap<String, Dimension>,
    damage_type: IndexMap<String, DamageType>,
    cat_variant: IndexMap<String, CatVariant>,
    chicken_variant: IndexMap<String, ChickenVariant>,
    cow_variant: IndexMap<String, CowVariant>,
    frog_variant: IndexMap<String, FrogVariant>,
    pig_variant: IndexMap<String, PigVariant>,
    wolf_sound_variant: IndexMap<String, WolfSoundVariant>,
    banner_pattern: IndexMap<String, BannerPattern>,
    enchantment: IndexMap<String, Enchantment>,
    pub jukebox_song: IndexMap<String, JukeboxSong>,
    instrument: IndexMap<String, Instrument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataPool<T> {
    data: T,
    weight: i32,
}

// TODO: remove in favor of numerical registry ids for `minecraft:dimension_type`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VanillaDimensionType {
    Overworld,
    OverworldCaves,
    TheEnd,
    TheNether,
}

impl VanillaDimensionType {
    pub fn resource_location(&self) -> ResourceLocation {
        match self {
            Self::Overworld => ResourceLocation::vanilla("overworld"),
            Self::OverworldCaves => ResourceLocation::vanilla("overworld_caves"),
            Self::TheEnd => ResourceLocation::vanilla("the_end"),
            Self::TheNether => ResourceLocation::vanilla("the_nether"),
        }
    }

    pub fn from_resource_location_string(resource_location: &str) -> Option<Self> {
        match resource_location {
            "minecraft:overworld" => Some(Self::Overworld),
            "minecraft:overworld_caves" => Some(Self::OverworldCaves),
            "minecraft:the_end" => Some(Self::TheEnd),
            "minecraft:the_nether" => Some(Self::TheNether),
            _ => None,
        }
    }
}

impl Registry {
    pub fn get_synced() -> Vec<Self> {
        let registry_entries = SYNCED_REGISTRIES
            .biome
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let biome = Registry {
            registry_id: ResourceLocation::vanilla("worldgen/biome"),
            registry_entries,
        };

        let mut registry_entries: Vec<RegistryEntry> = SYNCED_REGISTRIES
            .chat_type
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();

        // Custom RAW registry type that removes the vanilla sender name. Allows custom formatting.
        registry_entries.push(RegistryEntry::from_nbt_custom(
            "raw",
            &ChatType {
                chat: Decoration {
                    translation_key: "%s".to_string(),
                    parameters: vec!["content".to_string()],
                    style: None,
                },
                narration: Decoration {
                    translation_key: "%s says %s".to_string(),
                    parameters: vec!["sender".to_string(), "content".to_string()],
                    style: None,
                },
            },
        ));

        let chat_type = Registry {
            registry_id: ResourceLocation::vanilla("chat_type"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .wolf_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let wolf_variant = Registry {
            registry_id: ResourceLocation::vanilla("wolf_variant"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .cat_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let cat_variant = Registry {
            registry_id: ResourceLocation::vanilla("cat_variant"),
            registry_entries,
        };
        let registry_entries = SYNCED_REGISTRIES
            .chicken_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let chicken_variant = Registry {
            registry_id: ResourceLocation::vanilla("chicken_variant"),
            registry_entries,
        };
        let registry_entries = SYNCED_REGISTRIES
            .cow_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let cow_variant = Registry {
            registry_id: ResourceLocation::vanilla("cow_variant"),
            registry_entries,
        };
        let registry_entries = SYNCED_REGISTRIES
            .frog_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let frog_variant = Registry {
            registry_id: ResourceLocation::vanilla("frog_variant"),
            registry_entries,
        };
        let registry_entries = SYNCED_REGISTRIES
            .pig_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let pig_variant = Registry {
            registry_id: ResourceLocation::vanilla("pig_variant"),
            registry_entries,
        };
        let registry_entries = SYNCED_REGISTRIES
            .wolf_sound_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let wolf_sound_variant = Registry {
            registry_id: ResourceLocation::vanilla("wolf_sound_variant"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .painting_variant
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let painting_variant = Registry {
            registry_id: ResourceLocation::vanilla("painting_variant"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .dimension_type
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let dimension_type = Registry {
            registry_id: ResourceLocation::vanilla("dimension_type"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .damage_type
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let damage_type = Registry {
            registry_id: ResourceLocation::vanilla("damage_type"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .banner_pattern
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let banner_pattern = Registry {
            registry_id: ResourceLocation::vanilla("banner_pattern"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .jukebox_song
            .iter()
            .map(|(name, nbt)| RegistryEntry::from_nbt(name, nbt))
            .collect();
        let jukebox_song = Registry {
            registry_id: ResourceLocation::vanilla("jukebox_song"),
            registry_entries,
        };

        vec![
            cat_variant,
            chicken_variant,
            cow_variant,
            frog_variant,
            pig_variant,
            biome,
            chat_type,
            // trim_pattern,
            // trim_material,
            wolf_variant,
            painting_variant,
            wolf_sound_variant,
            dimension_type,
            damage_type,
            banner_pattern,
            // enchantment,
            jukebox_song,
            // instrument,
        ]
    }
}
