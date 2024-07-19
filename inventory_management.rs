use std::sync::{Arc, Mutex};
use std::thread;
use crate::rabbitmq::receive_message;
use crate::order_processing::Order;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;    
use scheduled_thread_pool::ScheduledThreadPool;

#[derive(Debug, Clone)]
pub struct Item {
    code: String,
    name: String,
    quantity: i32,
    entry: i32,
    exit: i32,
}

#[derive(Debug, Clone)]
struct Rack {
    name: String,
    capacity: i32,
    max_capacity: i32
}

pub struct InventoryManagement {
    inventory: Arc<Mutex<Vec<Item>>>,
    racks: Arc<Mutex<Vec<Rack>>>,
    pool: ScheduledThreadPool,
}

impl InventoryManagement {
    pub fn new() -> Self {
        let inventory = vec![
            Item { code: "001".to_string(), name: "Table".to_string(), quantity: 500, entry: 0, exit: 0},
            Item { code: "002".to_string(), name: "Chair".to_string(), quantity: 500, entry: 0, exit: 0},
            Item { code: "003".to_string(), name: "Cupboard".to_string(), quantity: 500, entry: 0, exit: 0},
        ];

        let racks = vec![
            Rack { name: "Rack A".to_string(), capacity: 1000, max_capacity: 1000},
            Rack { name: "Rack B".to_string(), capacity: 500, max_capacity: 1000},
            Rack { name: "Rack C".to_string(), capacity: 0, max_capacity: 1000},
            Rack { name: "Rack D".to_string(), capacity: 0, max_capacity: 1000},
            Rack { name: "Rack E".to_string(), capacity: 0, max_capacity: 1000},
        ];

        InventoryManagement { 
            inventory: Arc::new(Mutex::new(inventory)),
            racks: Arc::new(Mutex::new(racks)),
            pool: ScheduledThreadPool::new(1),
        }
    }

    pub fn start(&self, queue: String, order_tx:Sender<Order>) {
        let inventory_clone = Arc::clone(&self.inventory);
        thread::spawn(move || {
            loop {
                let order_message = receive_message(&queue);
                match serde_json::from_str::<Order>(&order_message) {
                    Ok(d_order) => {
                        // Update inventory
                        let mut inventory = inventory_clone.lock().unwrap();
                        InventoryManagement::update_inventory(&mut inventory, &d_order, &order_tx);
                    }
                    Err(e) => eprintln!("Failed to deserialize order: {}", e),
                }
            }
        });
    }

    pub fn update_inventory(inventory: &mut Vec<Item>, order: &Order, order_tx: &Sender<Order>) {
        if let Some(item) = inventory.iter_mut().find(|i| i.code == order.code) {
            match order.order_type.as_str() {
                "supply" => {
                    item.entry += order.quantity;
                    item.quantity += order.quantity;
                }
                "offload" => {
                    item.exit += order.quantity;
                    item.quantity -= order.quantity;
                }
                _ => {
                    println!("Unknown order type: {:?}", order);
                    return;
                }
            }
            println!("*Inventory Updated* for {}: {:?}", item.name, item);
            order_tx.send(order.clone()).unwrap();
        } else {
            println!("Item not found for order: {:?}", order);
        }
    }

    pub fn check_inventory_stock(&self, order: &Order) -> bool {
        let inventory = self.inventory.lock().unwrap();
        if let Some(item) = inventory.iter().find(|i| i.code == order.code) {
            // Check if the available quantity is enough for the offload order
            if order.order_type == "offload" && item.quantity < order.quantity {
                println!("❌ Order Declined! Insufficient stock for order {}. Items Requested: {}. Inventory balance: {}",
                            order.index, order.quantity, item.quantity);
                return false;
            }
            true
        } else {
            println!("Item not found for order: {:?}", order);
            false
        }
    }

    pub fn storage_management(&self, order_rx: Receiver<Order>) {
        let racks_clone = Arc::clone(&self.racks);
        thread::spawn(move || {
            for order in order_rx {
                let mut racks = racks_clone.lock().unwrap();
                match order.order_type.as_str() {
                    "supply" => {
                        let mut remaining_quantity = order.quantity;
                        for rack in racks.iter_mut() {
                            let space_available = rack.max_capacity - rack.capacity;
                            if space_available >= remaining_quantity {
                                // If there is enough space in the current rack
                                rack.capacity += remaining_quantity;
                                println!("*Storage* {} Capacity: {}. Added {} boxes to {}", 
                                        rack.name, rack.capacity, remaining_quantity, rack.name);
                                break; // Break the loop as the supply has been placed
                            } else {
                                // If there is not enough space in the current rack
                                rack.capacity = rack.max_capacity; // Fill up the rack
                                remaining_quantity -= space_available; // Update the remaining quantity
                                println!("*Storage* {} Capacity: {}. Added {} boxes to {}", 
                                        rack.name, rack.capacity, space_available, rack.name);
                            }
                        }
                        println!();
                    }
                    "offload" => {
                        let mut remaining_quantity = order.quantity;
                        for rack in racks.iter_mut() {
                            let items_to_remove = rack.capacity.min(remaining_quantity);
                            rack.capacity -= items_to_remove;
                            remaining_quantity -= items_to_remove;
                            println!("*Storage* {} Capacity: {}. Offloaded {} boxes from {}",
                                    rack.name, rack.capacity, items_to_remove, rack.name);
                            if remaining_quantity == 0 {
                                break; // If all items are removed, break the loop
                            }
                        }
                        println!();
                    }
                    _ => println!("Unknown order type: {:?}", order.order_type),
                }
            }
        });
    }

    pub fn scheduled_inventory_report(&self) {
        let inventory_clone = Arc::clone(&self.inventory);
        let racks_clone = Arc::clone(&self.racks);
        self.pool.execute_at_fixed_rate(Duration::from_secs(0), Duration::from_secs(20), move || {
            let inventory = inventory_clone.lock().unwrap();
            let racks = racks_clone.lock().unwrap();
            println!();
            println!("📝");
            println!("=========================== Inventory Report ============================");
            for item in inventory.iter() {
                println!("{:?}", item);
            }
            println!("============================== Rack Status ==============================");
            for rack in racks.iter() {
                println!("{:?}", rack);
            }
            println!("=========================================================================");
            println!();
        });
    }
}
                    
