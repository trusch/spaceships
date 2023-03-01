#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod rareships {
    use ink::prelude::string::String;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;

    use ink::storage::{Lazy, Mapping};
    use scale::{Decode, Encode};

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
        ShipAlreadyExists,
        NotEnoughEnergy,
        NotEnoughHealth,
        NotEnoughInventorySpace,
        NotEnoughCargoSpace,
        NotEnoughResources,
        NotShipOwner,
        NothingToSettle,
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
        id: ShipId,              // Unique identifier
        name: String,            // Name of the ship
        owner: AccountId,        // Owner of the ship
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
        position: (i32, i32),                // Position of the ship
        energy: u32,                         // Current energy of the ship
        health: u32,                         // Current health of the ship
        inventory: Vec<Item>,                // Inventory of the ship
        cargo: Vec<Item>,                    // Cargo of the ship
        orders: Vec<(Order, Option<Block>)>, // Orders of the ship and when they were started.
        last_recharge: Block,                // Block where the last recharge was settled
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
                last_recharge: Default::default(),
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
        Weapon(Weapon),     // Weapon item
        Armor(Armor),       // Armor item
        Resource(Resource), // Resource item
    }

    // Weapons are used to attack other ships or stations
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Weapon {
        id: ItemId,       // Unique identifier
        name: String,     // Name of the weapon
        damage: u32,      // Damage of the weapon
        range: u32,       // Range of the weapon
        energy_cost: u32, // Energy consumed by firing the weapon
    }

    // Armors are used to defend against attacks
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Armor {
        id: ItemId,   // Unique identifier
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
        id: ItemId,    // Unique identifier
        name: String,  // Name of the resource
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
    pub struct Rareships {
        ship_statics: Mapping<ShipId, ShipStatic>,
        ship_dynamics: Mapping<ShipId, ShipDynamic>,
        ship_ids: Lazy<Vec<ShipId>>,
    }

    impl Rareships {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
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
            if self.ship_statics.contains(ship_id) {
                return Err(Error::ShipAlreadyExists);
            }
            self.ship_statics.insert(
                ship_id,
                &ShipStatic {
                    id: ship_id,
                    name: String::from(""),
                    owner: self.env().caller(),
                    max_speed: 1000, // 1000 milli blocks per tile
                    max_inventory_size: 4,
                    max_cargo_size: 4,
                    max_energy: 100,
                    max_health: 100,
                    recharge_rate: 10,
                },
            );
            let mut ships = self.ship_ids.get_or_default();
            ships.push(ship_id);
            self.ship_ids.set(&ships);
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
            // check of ship exists
            if !self.ship_statics.contains(ship_id) {
                return Err(Error::ShipNotFound);
            }

            // get ship dynamic and static data
            let mut dynamic_data = self.ship_dynamics.get(ship_id).ok_or(Error::ShipNotFound)?;
            if dynamic_data.orders.is_empty() {
                return Ok(());
            }
            let static_data = self.ship_statics.get(ship_id).ok_or(Error::ShipNotFound)?;

            // recharge energy
            self.settle_recharge(&mut dynamic_data, &static_data)?;

            // settle orders order
            self.settle_top_order(&mut dynamic_data, &static_data)?;

            // save updated dynamics
            self.ship_dynamics.insert(ship_id, &dynamic_data);
            Ok(())
        }

        fn settle_recharge(
            &self,
            dynamic_data: &mut ShipDynamic,
            static_data: &ShipStatic,
        ) -> Result<(), Error> {
            let block = self.env().block_number();
            let elapsed = (block - dynamic_data.last_recharge);
            if elapsed > 0 {
                let amount = elapsed * static_data.recharge_rate;
                dynamic_data.energy = (dynamic_data.energy + amount) % static_data.max_energy;
                dynamic_data.last_recharge = block;
            }
            Ok(())
        }

        fn settle_top_order(
            &self,
            dynamic_data: &mut ShipDynamic,
            static_data: &ShipStatic,
        ) -> Result<(), Error> {
            if dynamic_data.orders.is_empty() {
                return Ok(())
            }
            match dynamic_data.orders.first().ok_or(Error::InvalidOrder)? {
                (Order::Move((direction, speed, distance)), Some(start)) => self.settle_movement(
                    dynamic_data,
                    static_data,
                    direction.clone(),
                    *speed,
                    *distance,
                    *start,
                )?,
                _ => return Err(Error::InvalidOrder),
            };
            Ok(())
        }

        fn settle_movement(
            &self,
            ship: &mut ShipDynamic,
            static_data: &ShipStatic,
            direction: Direction,
            speed: i32,    // milli blocks per tile
            distance: i32, // tiles
            start: Block,  // block number
        ) -> Result<(), Error> {
            let block = self.env().block_number();
            let elapsed = (block - start) as i32;
            if elapsed == 0 || elapsed * 1000 < speed {
                return Ok(())
            }
            let mut tiles_to_move = elapsed * 1000 / speed;
            if tiles_to_move > distance {
                tiles_to_move = distance;
            }

            let cost = move_energy_per_tile(speed, static_data.max_speed) as u32;
            if cost > ship.energy {
                return Err(Error::NotEnoughEnergy);
            }
            ship.energy = ship.energy - cost;

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
            let (x, y) = cube_coordinates_to_offset_coordinates((q, r, s));
            ship.position = (x % MAX_X, y % MAX_Y);

            let rest = distance - tiles_to_move;
            if rest == 0 {
                // order finished
                ship.orders.remove(0);
                if !ship.orders.is_empty() {
                    ship.orders[0].1 = Some(block);
                }
            } else {
                ship.orders[0] = (Order::Move((direction.clone(), speed, rest)), Some(block));
            }

            Ok(())
        }
    }

    fn offset_coordinates_to_cube_coordinates(c: (i32, i32)) -> (i32, i32, i32) {
        let (col, row) = c;
        let q = col - (row - (row & 1i32)) / 2;
        let r = row;
        (q, r, -q - r)
    }

    fn cube_coordinates_to_offset_coordinates(c: (i32, i32, i32)) -> (i32, i32) {
        let (q, r, _) = c;
        let col = q + (r - (r & 1i32)) / 2;
        let row = r;
        (col, row)
    }

    fn move_energy_per_tile(speed: i32, max_speed: i32) -> i32 {
        // example: max_speed 100  milli blocks per tile
        //              speed 1000 milli blocks per tile
        // speed/max_speed == 10
        // max -> 1
        // => cost = BASE_PRICE / (speed/max_speed)
        100 / (speed / max_speed)
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
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (1, 2),
                (2, 2),
                (3, 2),
                (2, 3),
                (3, 3),
                (4, 3),
                (3, 4),
                (4, 4),
                (5, 4),
                (4, 5),
                (5, 5),
            ];
            for c in cases {
                let (q, r, s) = offset_coordinates_to_cube_coordinates(c);
                let c2 = cube_coordinates_to_offset_coordinates((q, r, s));
                assert_eq!(c, c2);
            }
        }
    }
}
