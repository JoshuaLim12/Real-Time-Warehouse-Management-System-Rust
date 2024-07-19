use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::Rng;
use crate::rabbitmq::{receive_message, send_message};
use crate::order_processing::Order;

struct Forklift {
    name: String,
    in_use: bool,
}

impl Forklift {
    fn new(name: &str) -> Self {
        Forklift {
            name: name.to_string(),
            in_use: false,
        }
    }
}

pub struct GoodsTransportation {
    forklifts: Vec<Arc<Mutex<Forklift>>>,
}

impl GoodsTransportation {
    pub fn new() -> Self {
        let forklift_names = vec!["Forklift A", "Forklift B", "Forklift C"];
        let mut forklifts = Vec::new();
        for name in forklift_names {
            forklifts.push(Arc::new(Mutex::new(Forklift::new(name))));
        }
        GoodsTransportation { forklifts }
    }

    pub fn start(&self, queue: String) {
        let forklifts = self.forklifts.clone();
        let mut next_forklift_index = Arc::new(Mutex::new(0));
        thread::spawn(move || {
            loop {
                let order_message = receive_message(&queue); // Start listening to the queue and always ready for work
                match serde_json::from_str::<Order>(&order_message) {
                    Ok(d_order) => {
                        let mut forklift_found = false;
                        let forklifts_count = forklifts.len();

                        for _ in 0..forklifts_count {   // Try to find an available forklift in a separate thread
                            let forklift_index;
                            {
                                let mut index_guard = next_forklift_index.lock().unwrap();
                                forklift_index = *index_guard;
                                *index_guard = (*index_guard + 1) % forklifts_count;
                            }
                            let forklift = Arc::clone(&forklifts[forklift_index]);
                            let forklift_available = {
                                let mut f = forklift.lock().unwrap();
                                if !f.in_use {
                                    f.in_use = true;
                                    let forklift_name = f.name.clone();
                                    match d_order.order_type.as_str() {
                                        "supply" => {
                                            println!(
                                                "...transporting Supply Order {}: {} is transporting {} boxes of {} from Receiving Area to Storage",
                                                d_order.index, forklift_name, d_order.quantity, d_order.code
                                            );
                                        }
                                        "offload" => {
                                            println!(
                                                "...transporting Offload Order {}: {} is transporting {} boxes of {} from Storage to Shipping Area",
                                                d_order.index, forklift_name, d_order.quantity, d_order.code
                                            );
                                        }
                                        _ => println!("Unknown order type: {:?}", d_order),
                                    }
                                    true
                                } else {
                                    false
                                }
                            };

                            if forklift_available {
                                let d_order_clone = d_order.clone();
                                let forklift_name = {
                                    let f = forklift.lock().unwrap();
                                    f.name.clone()
                                };
                                
                                thread::spawn(move || {     // New thread created to handle each Order individually
                                    let mut rng = rand::thread_rng();       // Simulate transportation delay
                                    thread::sleep(Duration::from_secs(rng.gen_range(5..8)));
                                    println!();
                                    match d_order_clone.order_type.as_str() {
                                        "supply" => {
                                            println!(
                                                "(✅ Completed!) Supply Order {}: {} boxes of item {} has reached Storage using {}",
                                                d_order_clone.index, d_order_clone.quantity, d_order_clone.code, forklift_name
                                            );
                                        }
                                        "offload" => {
                                            println!(
                                                "(✅ Completed! ) Offload Order {}): {} boxes of item {} has reached Shipping Area using {}",
                                                d_order_clone.index, d_order_clone.quantity, d_order_clone.code, forklift_name
                                            );
                                        }
                                        _ => println!("Unknown order type: {:?}", d_order),
                                    }
                                    let s_order = serde_json::to_string(&d_order_clone).expect("Failed to serialize order");
                                    if let Err(e) = send_message("transport_queue", &s_order) {
                                        eprintln!("Failed to send message: {}", e);
                                    }
                                    let mut f = forklift.lock().unwrap();
                                    f.in_use = false;
                                });     // Thread will be destroyed and cleaned automatically by Rust runtime after finish

                                forklift_found = true;
                                next_forklift_index = Arc::new(Mutex::new((forklift_index + 1) % forklifts_count)); // Update the next forklift index
                                break;
                            } else {
                                next_forklift_index = Arc::new(Mutex::new((forklift_index + 1) % forklifts_count)); // Update the next forklift index
                            }
                        }
                        if !forklift_found {
                            println!("No available forklift for order");
                        }
                    }
                    Err(e) => eprintln!("Failed to deserialize order: {}", e),
                }
            }
        });
    }
}

