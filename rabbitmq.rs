use amiquip::{Connection, Exchange, Publish, Result, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};

pub fn send_message(queue_addr: &str, msg: &str) -> Result<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();
    let channel = connection.open_channel(None).unwrap();
    let exchange = Exchange::direct(&channel);
    exchange.publish(Publish::new(msg.as_bytes(), queue_addr)).unwrap();
    connection.close().unwrap();
    Ok(())
}

pub fn receive_message(queue_name: &str) -> String {
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
