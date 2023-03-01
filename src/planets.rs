use ink::primitives::AccountId;
use ink::prelude::{vec, vec::Vec};

type PlanetId = u32;

pub enum ResourceType {
    Iron,
    Copper,
    Silver,
    Gold,
    Uranium,    
}

pub struct Planet {
    id: u32,
    position: (u32, u32),
    owner: Option<AccountId>,
    resources: Vec<ResourceType>,
    mining_rates: Vec<u32>,
}


