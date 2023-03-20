#![cfg_attr(not(feature = "std"), no_std)]

mod inventory;
mod planets;

#[ink::contract]
mod rareships {
    use ink::prelude::string::{String, ToString};
    use ink::prelude::vec::Vec;

    use ink::storage::{Lazy, Mapping};
    use scale::{Decode, Encode};

    use crate::inventory::{Inventory, Item, Resource, ResourceType};
    use crate::planets::{Planet, PlanetId, PlanetLevel};

    const MAX_X: i32 = 10000;
    const MAX_Y: i32 = 10000;

    type ShipId = u32;
    type Speed = i32;
    type Distance = i32;
    type Block = u32;
    type Duration = u32;

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
        PlanetAlreadyExists,
        NotAuthorized,
        PlanetNotFound,
        ResourceNotFound,
        NotPlanetOwner,
    }

    impl From<crate::inventory::Error> for Error {
        fn from(error: crate::inventory::Error) -> Self {
            match error {
                crate::inventory::Error::InventoryFull => Error::NotEnoughInventorySpace,
            }
        }
    }

    // ship 
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Ship {
        // static data
        id: ShipId,              // Unique identifier
        name: String,            // Name of the ship
        owner: AccountId,        // Owner of the ship
        max_speed: i32,          // Max speed of the ship, milli-tiles per block
        max_inventory_size: u32, // Max size of the inventory
        max_cargo_size: u32,     // Max size of the cargo
        max_energy: u32,         // Max energy of the ship
        max_health: u32,         // Max health of the ship
        recharge_rate: u32,      // Energy recharge rate of the ship per block

        position: (i32, i32),                // Position of the ship
        energy: u32,                         // Current energy of the ship
        health: u32,                         // Current health of the ship
        inventory: Inventory,                // Inventory of the ship
        cargo: Inventory,                    // Cargo of the ship
        orders: Vec<(Order, Option<Block>)>, // Orders of the ship and when they were started.
        last_recharge: Block,                // Block where the last recharge was settled
    }

    // Orders are used to instruct what the ship should do next
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Order {
        Move((Direction, Speed, Distance)), // Move to in a direction
        Mine((PlanetId, ResourceType, Duration)),
    }

    // Directions are used to move the ship
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
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
        ships: Mapping<ShipId, Ship>,
        ship_ids: Lazy<Vec<ShipId>>,
        planets: Mapping<PlanetId, Planet>,
        admin: AccountId,
    }

    #[ink(event)]
    pub struct ShipSpawned {
        #[ink(topic)]
        ship_id: ShipId,
        #[ink(topic)]
        owner: AccountId,
    }

    #[ink(event)]
    pub struct ShipMoved {
        #[ink(topic)]
        ship_id: ShipId,
        #[ink(topic)]
        position: (i32, i32),
        energy_cost: u32,
    }

    #[ink(event)]
    pub struct ShipRecharged {
        #[ink(topic)]
        ship_id: ShipId,
        new_energy: u32,
    }

    #[ink(event)]
    pub struct EnergyUsed {
        #[ink(topic)]
        ship_id: ShipId,
        new_energy: u32,
    }

    #[ink(event)]
    pub struct OrderCreated {
        #[ink(topic)]
        ship_id: ShipId,
        order: Order,
    }

    #[ink(event)]
    pub struct OrderCompleted {
        #[ink(topic)]
        ship_id: ShipId,
        order: Order,
    }

    #[ink(event)]
    pub struct OrderUpdated {
        #[ink(topic)]
        ship_id: ShipId,
        order: Order,
    }

    #[ink(event)]
    pub struct ResourceMined {
        #[ink(topic)]
        ship_id: ShipId,
        planet_id: PlanetId,
        resource_type: ResourceType,
        quantity: u32,
    }

    #[ink(event)]
    pub struct DebugEvent {
        #[ink(topic)]
        value: String,
    }

    impl Rareships {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                ships: Mapping::new(),
                ship_ids: Default::default(),
                planets: Mapping::new(),
                admin: Rareships::env().caller(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        #[ink(message)]
        pub fn mint_planet(
            &mut self,
            planet_id: PlanetId,
            position: (i32, i32),
        ) -> Result<(), Error> {
            if self.env().caller() != self.admin {
                return Err(Error::NotAuthorized);
            }
            if self.planets.contains(planet_id) {
                return Err(Error::PlanetAlreadyExists);
            }
            self.planets.insert(
                planet_id,
                &Planet::new(planet_id, PlanetLevel::Basic, position),
            );
            Ok(())
        }

        #[ink(message)]
        pub fn spawn(&mut self, ship_id: ShipId) -> Result<(), Error> {
            if self.ships.contains(ship_id) {
                return Err(Error::ShipAlreadyExists);
            }
            self.ships.insert(
                ship_id,
                &Ship {
                    id: ship_id,
                    name: String::from(""),
                    owner: self.env().caller(),
                    max_speed: 10000, // 10000 milli tiles per block -> 10 tiles per block
                    max_inventory_size: 4,
                    max_cargo_size: 4,
                    max_energy: 1000,
                    max_health: 100,
                    recharge_rate: 10,
                    position: (0, 0),
                    energy: 1000,
                    health: 100,
                    inventory: Inventory::new(4),
                    cargo: Inventory::new(32),
                    orders: Vec::new(),
                    last_recharge: self.env().block_number(),
                },
            );
            let mut ships = self.ship_ids.get_or_default();
            ships.push(ship_id);
            self.ship_ids.set(&ships);
            self.env().emit_event(ShipSpawned {
                ship_id,
                owner: self.env().caller(),
            });
            Ok(())
        }

        #[ink(message)]
        pub fn order(&mut self, ship_id: ShipId, order: Order) -> Result<(), Error> {
            let ship_static = self.ships.get(ship_id).ok_or(Error::ShipNotFound)?;
            if ship_static.owner != self.env().caller() {
                return Err(Error::NotShipOwner);
            }
            let mut ship_dynamic = self.ships.get(ship_id).ok_or(Error::ShipNotFound)?;

            match &order {
                Order::Move((_, speed, distance)) => {
                    if *speed < 0 || *speed > ship_static.max_speed || *distance <= 0 {
                        return Err(Error::InvalidOrder);
                    }
                }
                Order::Mine((planet_id, resource_type, duration)) => {
                    if *duration <= 0 {
                        return Err(Error::InvalidOrder);
                    }
                    let planet = self.planets.get(*planet_id).ok_or(Error::PlanetNotFound)?;
                    if !planet.get_resources().contains(resource_type) {
                        return Err(Error::InvalidOrder);
                    }
                }
            }

            let start = match ship_dynamic.orders.is_empty() {
                true => Some(self.env().block_number()),
                false => None,
            };
            ship_dynamic.orders.push((order.clone(), start));
            self.ships.insert(ship_id, &ship_dynamic);
            self.env().emit_event(OrderCreated { ship_id, order });
            Ok(())
        }

        #[ink(message)]
        pub fn drop_order(&mut self, ship_id: ShipId, order_index: u32) -> Result<(), Error> {
            let mut ship = self.ships.get(ship_id).ok_or(Error::ShipNotFound)?;
            if ship.owner != self.env().caller() {
                return Err(Error::NotShipOwner);
            }
            if order_index >= ship.orders.len() as u32 {
                return Err(Error::InvalidOrder);
            }
            ship.orders.remove(order_index as usize);
            self.ships.insert(ship_id, &ship);
            Ok(())
        }

        #[ink(message)]
        pub fn settle(&mut self, ship_id: ShipId) -> Result<(), Error> {
            self.settle_ship(ship_id)?;
            Ok(())
        }

        #[ink(message)]
        pub fn settle_recharge_only(&mut self, ship_id: ShipId) -> Result<(), Error> {
            // get ship dynamic and static data
            let mut ship = self.ships.get(ship_id).ok_or(Error::ShipNotFound)?;

            // recharge energy
            self.settle_recharge(&mut ship)?;

            // save updated dynamics
            self.ships.insert(ship_id, &ship);

            Ok(())
        }

        #[ink(message)]
        pub fn get_ships(&self) -> Vec<ShipId> {
            self.ship_ids.get_or_default()
        }

        #[ink(message)]
        pub fn get_ship(&self, ship_id: ShipId) -> Option<Ship> {
            self.ships.get(ship_id)
        }

        #[ink(message)]
        pub fn get_planet(&self, planet_id: PlanetId) -> Option<Planet> {
            self.planets.get(planet_id)
        }

        pub fn settle_ship(&mut self, ship_id: ShipId) -> Result<(), Error> {
            // get ship dynamic and static data
            let mut ship = self.ships.get(ship_id).ok_or(Error::ShipNotFound)?;

            // recharge energy
            self.settle_recharge(&mut ship)?;

            // settle orders order
            self.settle_top_order(&mut ship)?;

            // save updated dynamics
            self.ships.insert(ship_id, &ship);
            Ok(())
        }

        fn settle_recharge(
            &self,
            ship: &mut Ship,
        ) -> Result<(), Error> {
            let block = self.env().block_number();
            let elapsed = block - ship.last_recharge;
            // self.debug(&format!("recharge: block: {} last: {} elapsed: {}", block, ship.last_recharge, elapsed));
            if elapsed > 0 && ship.energy < ship.max_energy {
                let amount = elapsed * ship.recharge_rate;
                let mut new_energy = ship.energy + amount;
                if new_energy > ship.max_energy {
                    new_energy = ship.max_energy;
                }
                ship.energy = new_energy;
                ship.last_recharge = block;
                self.env().emit_event(ShipRecharged {
                    ship_id: ship.id,
                    new_energy: ship.energy,
                });
            }
            Ok(())
        }

        fn settle_top_order(
            &self,
            ship: &mut Ship,
        ) -> Result<(), Error> {
            if ship.orders.is_empty() {
                return Ok(());
            }
            match ship.orders.first().ok_or(Error::InvalidOrder)? {
                (Order::Move((direction, speed, distance)), Some(start)) => self.settle_movement(
                    ship,
                    direction.clone(),
                    *speed,
                    *distance,
                    *start,
                )?,
                (Order::Mine((planet_id, resource_type, duration)), Some(start)) => self
                    .settle_mining(
                        ship,
                        *planet_id,
                        resource_type.clone(),
                        *duration,
                        *start,
                    )?,
                _ => return Err(Error::InvalidOrder),
            };
            Ok(())
        }

        fn settle_movement(
            &self,
            ship: &mut Ship,
            direction: Direction,
            speed: i32,    // milli tiles per block
            distance: i32, // tiles
            start: Block,  // block number
        ) -> Result<(), Error> {
            let block = self.env().block_number();
            let elapsed = (block - start) as i32;
            if elapsed == 0 || elapsed * speed < 1000 {
                return Ok(());
            }
            let mut tiles_to_move = elapsed * speed / 1000;
            if tiles_to_move > distance {
                tiles_to_move = distance;
            }
            if tiles_to_move <= 0 {
                return Ok(());
            }

            let cost = move_energy_per_tile(speed, ship.max_speed) as u32;
            if (cost as i32) * tiles_to_move > ship.energy as i32 {
                tiles_to_move = ship.energy as i32 / cost as i32;
            }
            ship.energy -= cost * tiles_to_move as u32;
            self.env().emit_event(EnergyUsed {
                ship_id: ship.id,
                new_energy: ship.energy,
            });

            // update the position by moving in direction tiles_to_move times
            let (q, r, s) = offset_coordinates_to_cube_coordinates(ship.position);
            let (q, r, s) = match direction {
                Direction::NorthWest => (q, r - tiles_to_move, s + tiles_to_move),
                Direction::NorthEast => (q + tiles_to_move, r - tiles_to_move, s),
                Direction::East => (q + tiles_to_move, r, s - tiles_to_move),
                Direction::SouthEast => (q, r + tiles_to_move, s - tiles_to_move),
                Direction::SouthWest => (q - tiles_to_move, r + tiles_to_move, s),
                Direction::West => (q - tiles_to_move, r, s + tiles_to_move),
            };
            let (mut x, mut y) = cube_coordinates_to_offset_coordinates((q, r, s));
            if x < 0 {
                x = MAX_X + x;
            }
            if y < 0 {
                y = MAX_Y + y;
            }
            ship.position = (x % MAX_X, y % MAX_Y);

            let rest = distance - tiles_to_move;
            if rest == 0 {
                // order finished
                let order = ship.orders.remove(0).0;
                if !ship.orders.is_empty() {
                    ship.orders[0].1 = Some(block);
                }
                self.env().emit_event(OrderCompleted {
                    ship_id: ship.id,
                    order: order,
                });
            } else {
                let order = Order::Move((direction, speed, rest));
                ship.orders[0] = (order.clone(), Some(block));
                self.env().emit_event(OrderUpdated {
                    ship_id: ship.id,
                    order: order,
                });
            }

            self.env().emit_event(ShipMoved {
                ship_id: ship.id,
                position: ship.position,
                energy_cost: cost * tiles_to_move as u32,
            });

            Ok(())
        }

        fn settle_mining(
            &self,
            ship: &mut Ship,
            planet_id: PlanetId,
            resource_type: ResourceType,
            duration: Block,
            start: Block,
        ) -> Result<(), Error> {
            let block = self.env().block_number();
            let elapsed = block - start;
            if elapsed < duration {
                // not enough time has passed
                return Ok(());
            }
            let cost = mine_energy_per_block() * duration;
            if cost > ship.energy {
                // not enough energy
                return Ok(());
            }
            let planet = self.planets.get(&planet_id).ok_or(Error::PlanetNotFound)?;
            if planet.get_position() != ship.position {
                // ship is not on the planet
                return Err(Error::ResourceNotFound);
            }
            if let Some(owner) = planet.get_owner() {
                if owner != ship.owner {
                    // planet is not owned by the ship's owner
                    return Err(Error::NotPlanetOwner);
                }
            }
            if !planet.get_resources().contains(&resource_type) {
                // planet does not have the resource
                return Err(Error::ResourceNotFound);
            }

            // extract the resource and put to the ship's inventory
            ship.energy -= cost;
            self.env().emit_event(EnergyUsed {
                ship_id: ship.id,
                new_energy: ship.energy,
            });
            let amount = planet.get_mining_rate(&resource_type) * duration as u32;
            ship.cargo
                .add_item(Item::Resource(Resource::new(resource_type.clone(), amount)))?;
            self.env().emit_event(ResourceMined {
                ship_id: ship.id,
                planet_id: planet_id,
                resource_type: resource_type.clone(),
                quantity: amount,
            });

            // order finished, remove it
            let order = ship.orders.remove(0).0;
            if !ship.orders.is_empty() {
                ship.orders[0].1 = Some(block);
            }
            self.env().emit_event(OrderCompleted {
                ship_id: ship.id,
                order: order,
            });

            Ok(())
        }

        fn debug(&self, msg: &str) {
            self.env().emit_event(DebugEvent {
                value: msg.to_string(),
            });
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
        100 * speed / max_speed
    }

    fn mine_energy_per_block() -> u32 {
        100
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
