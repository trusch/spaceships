use ink_prelude::vec::Vec;
use ink_prelude::borrow::ToOwned;

pub type ItemId = u32;

#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum ResourceType {
    Iron,
    Copper,
    Silver,
    Gold,
    Uranium,    
}

// Items are either something in the inventory or in the cargo
// They can be either a weapon, an armor or a resource (stack)
#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Item {
    Weapon(Weapon),     // Weapon item
    Armor(Armor),       // Armor item
    Resource(Resource), // Resource item
}

// Weapons are used to attack other ships or stations
#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Weapon {
    id: ItemId,       // Unique identifier
    damage: u32,      // Damage of the weapon
    range: u32,       // Range of the weapon
    energy_cost: u32, // Energy consumed by firing the weapon
}

// Armors are used to defend against attacks
#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Armor {
    id: ItemId,   // Unique identifier
    defense: u32, // Defense of the armor
}

// Resources are used to craft items
#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Resource {
    id: ItemId,          // Unique identifier
    resource_type: ResourceType, // Unique identifier
    quantity: u32,               // Quantity of the resource
}

#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Inventory {
    items: Vec<Item>,
    max_size: u32,
}

impl Item {
    pub fn id(&self) -> ItemId {
        match self {
            Item::Weapon(weapon) => weapon.id,
            Item::Armor(armor) => armor.id,
            Item::Resource(resource) => resource.id,
        }
    }
}

pub enum Error {
    InventoryFull,
}

impl Resource {
    pub fn new(resource_type: ResourceType, quantity: u32) -> Self {
        Self { id: 0, resource_type, quantity }
    }
}

impl Inventory {
    pub fn new(max_size: u32) -> Self {
        Self { items: Vec::new(), max_size }
    }

    // add_item adds the item and stacks it if possible
    // max_size is respected 
    // only resources are stackable
    // max stack size for resources is 64
    pub fn add_item(&mut self, item: Item) -> Result<(), Error>{
        if self.items.len() >= self.max_size as usize {
            return Err(Error::InventoryFull);
        }
        let mut item = item;
        if let Item::Resource(resource) = &mut item {
            let mut found = false;
            for item in self.items.iter_mut() {
                if let Item::Resource(r) = item {
                    if r.resource_type == resource.resource_type {
                        r.quantity += resource.quantity;
                        if r.quantity > 64 {
                            resource.quantity = r.quantity - 64;
                            r.quantity = 64;
                        } else {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if !found {
                self.items.push(Item::Resource(resource.to_owned()));
            }
        } else {
            self.items.push(item)
        }
        Ok(())
    }

}

