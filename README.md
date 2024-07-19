# Warehouse Management System Simulation

## Introduction

This project aims to simulate a Warehouse Management System (WMS) using Rust. The focus is on leveraging real-time computing concepts to manage inventory, process orders, and ensure efficient communication between various components. The goal is to provide a robust and high-performance solution to common challenges in warehouse operations.

## Project Structure

The project consists of the following key components:

- **Order Generation**: Simulates the creation of orders, including both supply and offload operations.
- **Inventory Management**: Manages the inventory updates based on incoming orders and checks stock levels.
- **Message Handling**: Uses RabbitMQ to simulate the sending and receiving of messages for order processing.
- **Scheduled Reporting**: Periodically generates inventory reports to provide real-time visibility into the warehouse status.

## Setup

### Prerequisites

- Rust (latest stable version)
- RabbitMQ server

### Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/your-username/wms-simulation.git
    cd wms-simulation
    ```

2. Install dependencies:
    ```sh
    cargo build
    ```

3. Start the RabbitMQ server:
    ```sh
    sudo service rabbitmq-server start
    ```

### Running the Simulation

To run the simulation, execute the following command:
```sh
cargo run
```

## Benchmarking

### Using Criterion

This project uses the `criterion` crate for benchmarking various components. The benchmarks include:

- **Order Generation**
- **Inventory Check**
- **Message Sending**
- **Message Receiving**

To run the benchmarks, use:
```sh
cargo bench
```

### Using BMA Benchmark

Additionally, the `bma_benchmark` crate is used to measure the latency of the following functions:

- `order_generation_benchmark`
- `inventory_check_benchmark`
- `message_sending_benchmark`
- `message_receiving_benchmark`

## Code Overview

### Order Generation
Handles the creation of supply and offload orders with random parameters to simulate real-world scenarios.

### Inventory Management
Manages inventory updates and checks stock levels based on incoming orders.

### Message Handling
Uses RabbitMQ to handle the sending and receiving of messages for order processing.

### Scheduled Reporting
Generates periodic inventory reports to provide real-time visibility into warehouse operations.

## Disclaimer

This project was intended to be a collaborative effort. However, due to unforeseen circumstances, I was unable to compare my work with my teammate. Therefore, only my benchmark results are included in this documentation.

## Conclusion

This project demonstrates how Rust, with its performance, safety, and concurrency features, can be utilized to develop efficient Warehouse Management Systems. The simulation provides insights into optimizing warehouse operations and tackling common challenges in inventory management and order processing.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

---

Feel free to contribute, raise issues, or suggest improvements to further enhance this simulation. Happy coding!
