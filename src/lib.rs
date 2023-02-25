#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod spaceships {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::prelude::vec;
    
    use ink::storage::{
        Mapping,
        Lazy,
    };
    use scale::{Encode, Decode};

    const MAX_X: i32 = 10000;
    const MAX_Y: i32 = 10000;

    type ShipId = u32;
    type ItemId = u32;
    type Speed = i32;
    type Distance = i32;
    type Block = u32;
    
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ShipNotFound,
        NotEnoughEnergy,
        NotEnoughHealth,
        NotEnoughInventorySpace,
        NotEnoughCargoSpace,
        NotEnoughResources,
        NotShipOwner,
        InvalidOrder,
    }

    // Ship static data
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct ShipStatic {
        // static data
        id: ShipId, // Unique identifier
        name: String, // Name of the ship
        owner: AccountId, // Owner of the ship
        max_speed: i32, // Max speed of the ship, in blocks needed per tile (-> lower is faster)
        max_inventory_size: u32, // Max size of the inventory
        max_cargo_size: u32, // Max size of the cargo
        max_energy: u32, // Max energy of the ship
        max_health: u32, // Max health of the ship
        recharge_rate: u32, // Energy recharge rate of the ship per block
    }

    // ship dynamic data
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct ShipDynamic {
        position: (i32, i32), // Position of the ship        
        energy: u32, // Current energy of the ship
        health: u32, // Current health of the ship
        inventory: Vec<Item>, // Inventory of the ship
        cargo: Vec<Item>, // Cargo of the ship
        orders: Vec<(Order, Option<Block>)>, // Orders of the ship and when they were started.
    }

    impl Default for ShipDynamic {
        fn default() -> Self {
            Self {
                position: (MAX_X / 2, MAX_Y / 2),
                energy: 100,
                health: 100,
                inventory: vec![],
                cargo: vec![],
                orders: vec![],
            }
        }
    }

    // Items are either something in the inventory or in the cargo
    // They can be either a weapon, an armor or a resource (stack)
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Item {
        Weapon(Weapon), // Weapon item
        Armor(Armor), // Armor item
        Resource(Resource), // Resource item
    }

    // Weapons are used to attack other ships or stations
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Weapon {
        id: ItemId, // Unique identifier
        name: String, // Name of the weapon
        damage: u32, // Damage of the weapon
        range: u32, // Range of the weapon
        energy_cost: u32, // Energy consumed by firing the weapon
    }

    // Armors are used to defend against attacks
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Armor {
        id: ItemId, // Unique identifier
        name: String, // Name of the armor
        defense: u32, // Defense of the armor
    }

    // Resources are used to craft items
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Resource {
        id: ItemId, // Unique identifier
        name: String, // Name of the resource
        quantity: u32, // Quantity of the resource
    }


    // Orders are used to instruct what the ship should do next
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Order {
        Move((Direction, Speed, Distance)), // Move to in a direction
    }

    // Directions are used to move the ship
    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Direction {
        NorthWest,
        NorthEast,
        East,
        SouthEast,
        SouthWest,
        West,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Spaceships {
        ship_statics: Mapping<ShipId, ShipStatic>,
        ship_dynamics: Mapping<ShipId, ShipDynamic>,
        ship_ids: Lazy<Vec<ShipId>>,
    }

    impl Spaceships {

        #[ink(constructor)]
        pub fn new() -> Self {
            Self{
                ship_statics: Mapping::new(),
                ship_dynamics: Mapping::new(),
                ship_ids: Default::default(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn spawn(&mut self, ship_id: ShipId) -> Result<(), Error> {
            self.ship_statics.insert(ship_id, &ShipStatic {
                id: ship_id,
                name: String::from(""),
                owner: self.env().caller(),
                max_speed: 1, // 1 block per tile
                max_inventory_size: 4,
                max_cargo_size: 4,
                max_energy: 100,
                max_health: 100,
                recharge_rate: 10,
            });
            self.ship_ids.get_or_default().push(ship_id);
            Ok(())
        }

        #[ink(message)]
        pub fn order(&mut self, ship_id: ShipId, order: Order) -> Result<(), Error> {
            let ship_static = self.ship_statics.get(ship_id).ok_or(Error::ShipNotFound)?;
            if ship_static.owner != self.env().caller() {
                return Err(Error::NotShipOwner);
            }
            let mut ship_dynamic = self.ship_dynamics.get(ship_id).unwrap_or_default();
            let start = match ship_dynamic.orders.is_empty() {
                true => Some(self.env().block_number()),
                false => None,
            };
            ship_dynamic.orders.push((order, start));
            self.ship_dynamics.insert(ship_id, &ship_dynamic);
            Ok(())
        }

        #[ink(message)]
        pub fn settle(&mut self, ship_id: ShipId) -> Result<(), Error> {
            self.settle_ship(ship_id)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_ships(&self) -> Vec<ShipId> {
            self.ship_ids.get_or_default()
        }

        #[ink(message)]
        pub fn get_ship_static(&self, ship_id: ShipId) -> Option<ShipStatic> {
            self.ship_statics.get(ship_id)
        }
        
        #[ink(message)]
        pub fn get_ship_dynamic(&self, ship_id: ShipId) -> Option<ShipDynamic> {
            self.ship_dynamics.get(ship_id)
        }

        pub fn settle_ship(&mut self, ship_id: ShipId) -> Result<(), Error> {
            if ! self.ship_statics.contains(ship_id) {
                return Err(Error::ShipNotFound);
            }
            let mut ship = self.ship_dynamics.get(ship_id).ok_or(Error::ShipNotFound)?;
            if ship.orders.is_empty() {
                return Ok(());
            }
            match ship.orders.first().ok_or(Error::InvalidOrder)? {
                (Order::Move(_), _) => self.settle_movement(&mut ship)?,
            };
            self.ship_dynamics.insert(ship_id, &ship);
            Ok(())
        }

        fn settle_movement(&mut self, ship: &mut ShipDynamic) -> Result<(), Error> {
            let (first_order, start) = ship.orders.first().ok_or(Error::InvalidOrder)?;
            let (direction, speed, distance) = match first_order {
                Order::Move((direction, speed, distance)) => (direction, speed, distance),
                _ => return Err(Error::InvalidOrder),
            };
            let start = match start {
                Some(start) => *start,
                None => return Err(Error::InvalidOrder),
            };
            let block = self.env().block_number();
            let elapsed = (block - start) as i32;
            if elapsed == 0 || elapsed < *speed {
                return Ok(()); // Nothing to do
            }
            let mut tiles_to_move = elapsed / speed;
            if tiles_to_move > *distance {
                tiles_to_move = *distance;
            }
            let (q, r, s) = offset_coordinates_to_cube_coordinates(ship.position);
            // update the position by moving in direction tiles_to_move times
            let (q, r, s) = match direction {
                Direction::NorthWest => (q, r - tiles_to_move, s + tiles_to_move),
                Direction::NorthEast => (q + tiles_to_move, r - tiles_to_move, s),
                Direction::East => (q + tiles_to_move, r, s - tiles_to_move),
                Direction::SouthEast => (q, r + tiles_to_move, s - tiles_to_move),
                Direction::SouthWest => (q - tiles_to_move, r + tiles_to_move, s),
                Direction::West => (q - tiles_to_move, r, s + tiles_to_move),
            };
            let (x,y) = cube_coordinates_to_offset_coordinates((q, r, s));
            ship.position = (x % MAX_X, y % MAX_Y);

            let rest = distance - tiles_to_move;
            if rest == 0 {
                // order finished
                ship.orders.remove(0);
                if !ship.orders.is_empty() {
                    ship.orders[0].1 = Some(block);
                }
            } else {
                ship.orders[0] = (Order::Move((direction.clone(), *speed, rest)), Some(block));
            }

            Ok(())
        }
    }

    fn offset_coordinates_to_cube_coordinates(c: (i32, i32)) -> (i32, i32, i32) {
        let (col, row) = c;
        let q = col - (row - (row & 1i32)) / 2;
        let r = row;
        (q, r, -q-r)
    }

    fn cube_coordinates_to_offset_coordinates(c: (i32, i32, i32)) -> (i32, i32) {
        let (q, r, _) = c;
        let col = q + (r - (r & 1i32)) / 2;
        let row = r;
        (col, row)
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        #[ink::test]
        fn coordinates_calculation_works() {
            let cases = vec![
                (0,0),
                (1,0),
                (0,1),
                (1,1),
                (2,1),
                (1,2),
                (2,2),
                (3,2),
                (2,3),
                (3,3),
                (4,3),
                (3,4),
                (4,4),
                (5,4),
                (4,5),
                (5,5),
            ];
            for c in cases {
                let (q, r, s) = offset_coordinates_to_cube_coordinates(c);
                let c2 = cube_coordinates_to_offset_coordinates((q, r, s));
                assert_eq!(c, c2);
            }
            
        }
    }
}
