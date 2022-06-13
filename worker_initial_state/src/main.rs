use amiquip::{Channel, ConsumerMessage, QueueDeclareOptions};
use constants::queues::{
    AVG_TO_FILTER_SCORE, QUEUE_COMMENTS_TO_FILTER_STUDENTS, QUEUE_COMMENTS_TO_GROUP_BY,
    QUEUE_COMMENTS_TO_JOIN, QUEUE_COMMENTS_TO_MAP, QUEUE_INITIAL_STATE, QUEUE_POSTS_TO_AVG,
    QUEUE_POSTS_TO_FILTER_SCORE, QUEUE_POSTS_TO_GROUP_BY, QUEUE_POSTS_TO_JOIN,
};
use handlers::handle_comments::handle_comments;
use handlers::handle_posts::handle_posts;
use handlers::handle_posts_end::handle_post_end;
use utils::{
    logger::logger_create,
    middleware::{
        middleware_connect, middleware_consumer_end, middleware_create_channel,
        middleware_create_consumer, middleware_create_exchange, middleware_declare_queue,
    },
};

mod constants;
mod entities;
mod handlers;
mod messages;
mod utils;

// msg opcodes
const OPCODE_POST: u8 = 0;
const OPCODE_POST_END: u8 = 1;
const OPCODE_COMMENT: u8 = 2;
const OPCODE_COMMENT_END: u8 = 3;

pub const LOG_LEVEL: &str = "debug";
pub const LOG_RATE: usize = 100000;

fn rabbitmq_declare_queues(channel: &Channel) {
    for queue in [
        AVG_TO_FILTER_SCORE,
        QUEUE_COMMENTS_TO_FILTER_STUDENTS,
        QUEUE_COMMENTS_TO_GROUP_BY,
        QUEUE_COMMENTS_TO_JOIN,
        QUEUE_COMMENTS_TO_MAP,
        QUEUE_INITIAL_STATE,
        QUEUE_POSTS_TO_AVG,
        QUEUE_POSTS_TO_FILTER_SCORE,
        QUEUE_POSTS_TO_GROUP_BY,
        QUEUE_POSTS_TO_JOIN,
    ] {
        channel
            .queue_declare(queue, QueueDeclareOptions::default())
            .unwrap();
    }
}

fn main() {
    let logger = logger_create();
    logger.info("start".to_string());

    let mut connection = middleware_connect(&logger);
    let channel = middleware_create_channel(&mut connection);
    rabbitmq_declare_queues(&channel);
    let queue = middleware_declare_queue(&channel, QUEUE_INITIAL_STATE);
    let consumer = middleware_create_consumer(&queue);
    let exchange = middleware_create_exchange(&channel);

    let mut n_post_received: usize = 0;
    let mut n_comment_received: usize = 0;

    let mut end = false;
    let mut n_end = 0;

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                let splited: Vec<&str> = body.split('|').collect();
                let opcode = splited[0].parse::<u8>().unwrap();
                let payload = splited[1..].join("|");

                match opcode {
                    OPCODE_POST_END => {
                        handle_post_end(&exchange, logger.clone());
                    }
                    OPCODE_COMMENT_END => {
                        if middleware_consumer_end(
                            &mut n_end,
                            &exchange,
                            [QUEUE_COMMENTS_TO_MAP].to_vec(),
                        ) {
                            end = true;
                        }
                    }
                    OPCODE_POST => {
                        handle_posts(
                            payload.to_string(),
                            &exchange,
                            &mut n_post_received,
                            logger.clone(),
                        );
                    }
                    OPCODE_COMMENT => {
                        handle_comments(
                            payload.to_string(),
                            &exchange,
                            &mut n_comment_received,
                            logger.clone(),
                        );
                    }
                    _ => logger.info("opcode invalid".to_string()),
                }

                consumer.ack(delivery).unwrap();

                if end {
                    break;
                }
            }
            _ => {}
        }
    }

    connection.close().unwrap();

    logger.info("shutdown".to_string());
}
