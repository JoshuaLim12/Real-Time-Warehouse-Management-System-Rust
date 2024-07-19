extern crate bma_benchmark;
use bma_benchmark::benchmark;
use criterion::{black_box, Criterion, criterion_group, criterion_main};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use rand::Rng;
use amiquip::{Connection, Exchange, Publish, Result, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    index: i32,
    code: String,
    quantity: i32,
    order_type: String, // "supply" or "offload"
}

struct InventoryManagement {
    inventory: Arc<Mutex<Vec<Item>>>,
}

impl InventoryManagement {
    pub fn new() -> Self {
        let inventory = vec![
            Item { code: "001".to_string(), name: "Table".to_string(), quantity: 500, entry: 0, exit: 0},
            Item { code: "002".to_string(), name: "Chair".to_string(), quantity: 500, entry: 0, exit: 0},
            Item { code: "003".to_string(), name: "Cupboard".to_string(), quantity: 500, entry: 0, exit: 0},
        ];
        InventoryManagement { 
            inventory: Arc::new(Mutex::new(inventory)),
        }
    }

    pub fn check_inventory_stock(&self, order: &Order) -> bool {
        let inventory = self.inventory.lock().unwrap();
        if let Some(item) = inventory.iter().find(|i| i.code == order.code) {
            if order.order_type == "offload" && item.quantity < order.quantity {
                return false;
            }
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    code: String,
    name: String,
    quantity: i32,
    entry: i32,
    exit: i32,
}

fn send_message(queue_addr: &str, msg: &str) -> Result<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();
    let channel = connection.open_channel(None).unwrap();
    let exchange = Exchange::direct(&channel);
    exchange.publish(Publish::new(msg.as_bytes(), queue_addr)).unwrap();
    connection.close().unwrap();
    Ok(())
}

fn receive_message(queue_name: &str) -> String {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();
    let channel = connection.open_channel(None).unwrap();
    let queue = channel.queue_declare(queue_name, QueueDeclareOptions::default()).unwrap();
    let consumer = queue.consume(ConsumerOptions::default()).unwrap();
    let mut msg = String::new();
    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                msg = String::from_utf8_lossy(&delivery.body).to_string();
                consumer.ack(delivery).unwrap();
                break;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }
    connection.close().unwrap();
    msg
}

fn order_generation_benchmark(c: &mut Criterion) {
    let inventory_management = Arc::new(InventoryManagement::new());
    c.bench_function("order_generation", |b| b.iter(|| {
        let mut rng = rand::thread_rng();
        let code = ["001", "002", "003"][rng.gen_range(0..3)].to_string();
        let quantity = rng.gen_range(100..500);
        let order_type = if rng.gen_bool(0.5) { "supply".to_string() } else { "offload".to_string() };
        let order = Order {
            index: rng.gen(),
            code,
            quantity,
            order_type,
        };
        black_box(order);
    }));
}

fn inventory_check_benchmark(c: &mut Criterion) {
    let inventory_management = Arc::new(InventoryManagement::new());
    let order = Order {
        index: 1,
        code: "001".to_string(),
        quantity: 100,
        order_type: "offload".to_string(),
    };
    c.bench_function("inventory_check", |b| b.iter(|| {
        black_box(inventory_management.check_inventory_stock(&order));
    }));
}

fn message_sending_benchmark(c: &mut Criterion) {
    let msg = "Test message";
    c.bench_function("send_message", |b| b.iter(|| {
        send_message("test_queue", msg).unwrap();
    }));
}

fn message_receiving_benchmark(c: &mut Criterion) {
    c.bench_function("receive_message", |b| b.iter(|| {
        receive_message("test_queue");
    }));
}

criterion_group!(benches, order_generation_benchmark, inventory_check_benchmark, message_sending_benchmark, message_receiving_benchmark);
criterion_main!(benches);


// extern crate bma_benchmark;
// use bma_benchmark::benchmark;
// use criterion::{black_box, Criterion, criterion_group, criterion_main};
// use std::sync::{Arc, Mutex};
// use std::sync::mpsc::{Sender, Receiver, channel};
// use std::thread;
// use std::time::Duration;
// use rand::Rng;
// use amiquip::{Connection, Exchange, Publish, Result, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
// use serde::{Serialize, Deserialize};
// use scheduled_thread_pool::ScheduledThreadPool;

// // Define your structs and implementations here...

// fn goods_transportation_benchmark(c: &mut Criterion) {
//     let goods_transportation = Arc::new(GoodsTransportation::new());
//     let queue = "test_queue".to_string();
//     c.bench_function("goods_transportation", |b| b.iter(|| {
//         black_box(goods_transportation.start(queue.clone()));
//     }));
// }

// fn inventory_update_benchmark(c: &mut Criterion) {
//     let inventory_management = Arc::new(InventoryManagement::new());
//     let order = Order {
//         index: 1,
//         code: "001".to_string(),
//         quantity: 100,
//         order_type: "supply".to_string(),
//     };
//     let order_tx = channel::<Order>().0;
//     c.bench_function("inventory_update", |b| b.iter(|| {
//         black_box(inventory_management.update_inventory(order.clone(), &order_tx));
//     }));
// }

// fn scheduled_inventory_report_benchmark(c: &mut Criterion) {
//     let inventory_management = Arc::new(InventoryManagement::new());
//     c.bench_function("scheduled_inventory_report", |b| b.iter(|| {
//         black_box(inventory_management.scheduled_inventory_report());
//     }));
// }

// criterion_group!(
//     benches,
//     goods_transportation_benchmark,
//     inventory_update_benchmark,
//     scheduled_inventory_report_benchmark
// );
// criterion_main!(benches);
