use super::logger::Logger;
use amiquip::{Channel, Connection, QueueDeclareOptions, Consumer, ConsumerOptions, Queue, Exchange};
use std::env;

/* Middleware */

fn get_rabbitmq_user() -> String {
    match env::var("RABBITMQ_USER") {
        Ok(value) => value,
        Err(_) => {
            panic!("could not get rabbitmq user from env")
        }
    }
}

fn get_rabbitmq_password() -> String {
    match env::var("RABBITMQ_PASSWORD") {
        Ok(value) => value,
        Err(_) => {
            panic!("could not get rabbitmq password from env")
        }
    }
}

fn get_n_producers() -> usize {
    let mut n_producers = 1;
    if let Ok(value) = env::var("N_PRODUCERS") {
        n_producers = value.parse::<usize>().unwrap();
    }
    n_producers
}

fn get_n_consumers() -> usize {
    let mut n_consumers = 1;
    if let Ok(value) = env::var("N_CONSUMERS") {
        n_consumers = value.parse::<usize>().unwrap();
    }
    n_consumers
}

pub fn rabbitmq_connect(logger: &Logger) -> Connection {
    let rabbitmq_user = get_rabbitmq_user();
    let rabbitmq_password = get_rabbitmq_password();

    let rabbitmq_connection;
    match Connection::insecure_open(
        &format!(
            "amqp://{}:{}@rabbitmq:5672",
            rabbitmq_user, rabbitmq_password
        )
        .to_owned(),
    ) {
        Ok(connection) => {
            logger.info("connected with rabbitmq".to_string());
            rabbitmq_connection = connection;
        }
        Err(_) => {
            panic!("could not connect with rabbitmq")
        }
    }

    rabbitmq_connection
}

pub fn rabbitmq_create_channel(connection: &mut Connection) -> Channel {
    connection.open_channel(None).unwrap()
}

pub fn rabbitmq_declare_queue<'a>(channel: &'a Channel, queue_name: &'a str) -> Queue<'a> {
    channel
        .queue_declare(queue_name, QueueDeclareOptions::default())
        .unwrap()
}

pub fn rabbitmq_create_consumer<'a>(queue: &'a Queue) -> Consumer<'a> {
    queue.consume(ConsumerOptions::default()).unwrap()
}

pub fn rabbitmq_create_exchange(channel: &Channel) -> Exchange {
    Exchange::direct(&channel)
}

pub fn rabbitmq_end_reached(n_end: &mut usize) -> bool {
    *n_end += 1;

    let n_producers = get_n_producers();

    *n_end == n_producers
}