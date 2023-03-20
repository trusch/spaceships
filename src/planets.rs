use ink::primitives::AccountId;
use ink::prelude::{vec, vec::Vec};

use crate::inventory::{ResourceType, Inventory};

pub type PlanetId = u32;


#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum PlanetLevel {
    Basic,
    Advanced,
    Fortress,
}

#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Planet {
    id: u32,
    level: PlanetLevel,
    position: (i32, i32),
    owner: Option<AccountId>,
    resources: Vec<ResourceType>,
    mining_rates: Vec<u32>,
    inventory: Inventory,
}

impl Planet {
    pub fn new(id: u32, level: PlanetLevel, position: (i32, i32)) -> Self {
        let resources = match level {
            PlanetLevel::Basic => vec![ResourceType::Iron],
            PlanetLevel::Advanced => vec![ResourceType::Iron, ResourceType::Copper],
            PlanetLevel::Fortress => vec![ResourceType::Iron, ResourceType::Copper, ResourceType::Silver],
        };

        let mining_rates = match level {
            PlanetLevel::Basic => vec![1],
            PlanetLevel::Advanced => vec![1, 1],
            PlanetLevel::Fortress => vec![1, 1, 1],
        };

        let inventory_size = match level {
            PlanetLevel::Basic => 1,
            PlanetLevel::Advanced => 3,
            PlanetLevel::Fortress => 5,
        };

        Self {
            id,
            level,
            position,
            owner: None,
            resources,
            mining_rates,
            inventory: Inventory::new(inventory_size),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_level(&self) -> PlanetLevel {
        self.level.clone()
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.position
    }

    pub fn get_owner(&self) -> Option<AccountId> {
        self.owner
    }

    pub fn get_resources(&self) -> Vec<ResourceType> {
        self.resources.clone()
    }

    pub fn get_mining_rates(&self) -> Vec<u32> {
        self.mining_rates.clone()
    }

    pub fn set_owner(&mut self, owner: AccountId) {
        self.owner = Some(owner);
    }

    pub fn get_mining_rate(&self, resource_type: &ResourceType) -> u32 {
        let mut mining_rate = 0;
        for (index, resource) in self.resources.iter().enumerate() {
            if resource == resource_type {
                mining_rate = self.mining_rates[index];
            }
        }
        mining_rate
    }
}


