extern crate sysfs_gpio;

use std::env;
use std::io::prelude::*;
use std::io::stdout;
use sysfs_gpio::{Direction, Edge, Pin};
use tokio::sync::mpsc::{channel};

use rumq_client::{self, MqttOptions, Publish, QoS, Request};
use std::time::Duration;

async fn interrupt(pin: u64) -> sysfs_gpio::Result<()> {
    let input = Pin::new(pin);
    let (mut requests_tx, _requests_rx) = channel(10);
    let mut mqttoptions = MqttOptions::new("water1", "192.168.1.2", 1883);
    mqttoptions.set_keep_alive(5).set_throttle(Duration::from_secs(1));

    input.with_exported(|| {

        input.set_direction(Direction::In)?;
        input.set_edge(Edge::BothEdges)?;
        let mut poller = input.get_poller()?;
        loop {
            match poller.poll(1000)? {
                Some(value) => {
                    println!("{}", value);
                    requests_tx.send(publish_request(3));
                    //.await.unwrap();
                },
                None => {
                    let mut stdout = stdout();
                    stdout.write_all(b".")?;
                    stdout.flush()?;
                }
            }
        }
    })
}

fn publish_request(i: u8) -> Request {
    let topic = "hello/world".to_owned();
    let payload = vec![1, 2, 3, i];

    let publish = Publish::new(&topic, QoS::AtLeastOnce, payload);
    Request::Publish(publish)
}

#[tokio::main(basic_scheduler)]

async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./interrupt <pin>");
    } else {
        //pretty_env_logger::init();
        //color_backtrace::install();

        match args[1].parse::<u64>() {
            Ok(pin) => match interrupt(pin).await {
                Ok(()) => println!("Interrupting Complete!"),
                Err(err) => println!("Error: {}", err),
            },
            Err(_) => println!("Usage: ./interrupt <pin>"),
        }
    }
}
