use kafka::producer::{Producer, Record, RequiredAcks};

use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::BaseRecord;
use rdkafka::producer::ProducerContext;
use rdkafka::producer::ThreadedProducer;
use rdkafka::ClientConfig;
use rdkafka::ClientContext;
use rdkafka::Message;

use serde_json;
use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

mod bench;
mod log_file;
mod ssh;

use log_file::LogEntry;
use log_file::LogFile;

// ~ 5 s
fn write_stdout(file: &mut LogFile) {
    file.for_each(|entry| {
        let json = serde_json::to_string(&entry).unwrap();
        println!("{}", json);
    });
}

// ~ 4.8 s
fn channel_write_stdout(file: &mut LogFile) {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        for entry in rx {
            let json = serde_json::to_string(&entry).unwrap();
            println!("{}", json);
        }
    });

    file.for_each(|entry| {
        tx.send(entry).unwrap();
    });

    drop(tx);
    handle.join().unwrap();
}

// ~ 56 s
fn naive_write_kafka(file: &mut LogFile) {
    let mut producer = Producer::from_hosts(vec!["gary.home:9092".to_owned()])
        .with_ack_timeout(Duration::from_millis(200))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();

    file.for_each(|entry| {
        let json = serde_json::to_string(&entry).unwrap();
        producer
            .send(&Record::from_key_value(
                "dhcp",
                entry.remote_id.as_bytes(),
                json.as_bytes(),
            ))
            .expect("Failed to send log entry to Kafka");
    });
}

// ~ 52 s
fn naive_channel_write_kafka(file: &mut LogFile) {
    let (tx, rx): (Sender<LogEntry>, Receiver<LogEntry>) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut producer = Producer::from_hosts(vec!["gary.home:9092".to_owned()])
            .with_ack_timeout(Duration::from_millis(200))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();

        for entry in rx {
            let json = serde_json::to_string(&entry).unwrap();
            producer
                .send(&Record::from_key_value(
                    "dhcp",
                    entry.remote_id.as_bytes(),
                    json.as_bytes(),
                ))
                .expect("Failed to send log entry to Kafka");
        }
    });

    file.for_each(|entry| {
        tx.send(entry).unwrap();
    });

    drop(tx);
    handle.join().unwrap();
}

struct Callback;
impl ClientContext for Callback {}
impl ProducerContext for Callback {
    type DeliveryOpaque = ();

    fn delivery(
        &self,
        delivery_result: &Result<BorrowedMessage<'_>, (KafkaError, BorrowedMessage<'_>)>,
        _: <Self as ProducerContext>::DeliveryOpaque,
    ) {
        let dr = delivery_result.as_ref();
        match dr {
            Ok(msg) => {
                // println!("Delivered message {:?}", msg.payload());
            }
            Err(e) => {
                // eprintln!("Failed to produce message {:?}", e);
            }
        }
    }
}

// ~ 6s
fn write_rdkafka(file: &mut LogFile<'_>) {
    let producer: ThreadedProducer<Callback> = ClientConfig::new()
        .set("bootstrap.servers", "gary.home:9092")
        .set("message.timeout.ms", "1000")
        .set("queue.buffering.max.messages", "500000")
        .create_with_context(Callback {})
        .expect("Could not create producer");

    file.for_each(|entry| {
        let json = serde_json::to_string(&entry).unwrap();
        let record = BaseRecord::to("dhcp").key(&entry.remote_id).payload(&json);
        producer.send(record).expect("Failed to produce message");
    });
}

// #[tokio::main]
fn main() {
    let contents = fs::read_to_string("./logs/dhcp-log-large.log").unwrap();
    let lines = contents.lines();
    let mut file = LogFile::new(lines);

    // write_stdout(&mut file);
    // channel_write_stdout(&mut file);
    // naive_write_kafka(&mut file);
    // naive_channel_write_kafka(&mut file);
    write_rdkafka(&mut file);
}
