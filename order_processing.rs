use std::thread;
use std::sync::Arc;
use std::time::Duration;
use rand::Rng;
use crate::inventory_management::InventoryManagement;
use crate::rabbitmq::send_message;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub index: i32,
    pub code: String,
    pub quantity: i32,
    pub order_type: String, // "supply" or "offload"
}

impl Order {
    pub fn start(inventory_management: Arc<InventoryManagement>,) {
        thread::spawn(move || {
            let mut index = 0;
            loop {                              
                let item_codes = vec!["001", "002", "003"]; // Start to generate random orders 
                let mut rng = rand::thread_rng();
                let code = item_codes[rng.gen_range(0..item_codes.len())].to_string();
                let quantity = rng.gen_range(100..500);
                let order_type = if rng.gen_bool(0.5) { "supply".to_string() } else { "offload".to_string() }; // 50% chance
                index += 1;

                let order = Order {
                    index,
                    code,
                    quantity,
                    order_type,
                };
                println!();
                
                match order.order_type.as_str() {   // Distribute orders accordingly
                    "supply" => {
                        println!("(🎁 Supply Received): {:?}", order);
                        //Send order to GTS to transport the goods via RMQ
                        let s_order = serde_json::to_string(&order).expect("Failed to serialize order");
                        if let Err(e) = send_message("order_queue", &s_order) {
                            eprintln!("Failed to send message: {}", e);
                        }
                    }
                    "offload" => {
                        println!("(🚛 Offload Requested): {:?}", order);
                        if inventory_management.check_inventory_stock(&order) { // Check Inventory First
                            // Send order to GTS to transport the goods via RMQ
                            let s_order = serde_json::to_string(&order).expect("Failed to serialize order");
                            if let Err(e) = send_message("order_queue", &s_order) {
                                eprintln!("Failed to send message: {}", e);
                            }
                        }
                    }
                    _ => println!("Unknown order type: {:?}", order), // Error handling
                }
                thread::sleep(Duration::from_secs(rng.gen_range(2..5)));    // Simulate random time delay between order generation
            }
        });
    }
}
