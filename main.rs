// Listing all the mods needed 
mod inventory_management;
mod order_processing;
mod goods_transportation;
mod rabbitmq;

//Listing all crates, functions and libraries needed
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::sync::Arc;
use crate::inventory_management::InventoryManagement;
use order_processing::Order;
use goods_transportation::GoodsTransportation;

fn main() {
    //Create channel for storage management
    let (order_tx, order_rx) = channel::<Order>();
    
    // Create all queues of RabbitMQ
    let order_queue: String = "order_queue".to_string();
    let transport_queue = "transport_queue".to_string();

    // Initiate the systems
    let inventory_management = Arc::new(InventoryManagement::new());
    let inventory_management_clone = Arc::clone(&inventory_management);
    let goods_transportation = Arc::new(GoodsTransportation::new());

    // Start the systems
    Order::start(inventory_management_clone);
    goods_transportation.start(order_queue);
    inventory_management.start(transport_queue,order_tx);
    inventory_management.storage_management(order_rx);
    inventory_management.scheduled_inventory_report();
    
    // Loop the main thread to keep the simulation running
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}   


