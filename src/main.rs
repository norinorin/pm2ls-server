extern crate cpal;
extern crate websocket;

mod decoder;

use cpal::platform::Device;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use decoder::OpusDecoder;
use ringbuf::RingBuffer;
use std::fmt;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use websocket::sync::Client;
use websocket::sync::Server;
use websocket::ws::dataframe::DataFrame;
use websocket::OwnedMessage;

struct DeviceWrapper {
    output_device: Device,
}

fn get_int<T>(client: &mut Client<TcpStream>) -> T
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Debug,
{
    String::from_utf8_lossy(&client.recv_message().unwrap().take_payload())
        .parse::<T>()
        .unwrap()
}

fn main() {
    let server = Server::bind("0.0.0.0:7619").unwrap();
    let data = Arc::new(Mutex::new(DeviceWrapper {
        output_device: cpal::default_host().default_output_device().unwrap(),
    }));

    for request in server.filter_map(Result::ok) {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            println!("Got a connection! Trying to accept...");

            let data_ref = match data.try_lock() {
                Ok(data_ref) => data_ref,
                _ => {
                    println!("Currently in use. Rejecting incoming request.",);
                    request.reject().unwrap();
                    return;
                }
            };

            let mut client = request.accept().unwrap();

            println!("Connection has been established.");

            let sample_rate: u32 = get_int(&mut client);
            let channels: u16 = get_int(&mut client);
            let buffer_size: u32 = get_int(&mut client);
            let ring_buffer_size = buffer_size as usize * channels as usize;

            println!("sample rate: {}", sample_rate);
            println!("channels: {}", channels);
            println!(
                "buffer size ({} msec latency): {}",
                buffer_size / (sample_rate / 1000),
                buffer_size
            );

            let config = cpal::StreamConfig {
                channels,
                sample_rate: cpal::SampleRate(sample_rate),
                buffer_size: cpal::BufferSize::Fixed(buffer_size),
            };
            // TODO: reuse decoder
            let decoder = OpusDecoder::new(sample_rate as i32, channels as i32).unwrap();
            let ring = RingBuffer::<f32>::new(ring_buffer_size * 2);
            let (mut producer, mut consumer) = ring.split();

            data_ref
                .output_device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let read = consumer.pop_slice(data);
                        data[read..].iter_mut().for_each(|s| *s = 0.0);
                    },
                    move |error: cpal::StreamError| eprintln!("Stream threw an error: {}", error),
                )
                .unwrap()
                .play()
                .unwrap();

            for message in client.incoming_messages() {
                if message.is_err() {
                    break;
                }

                let message = message.unwrap();

                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        client.send_message(&message).unwrap();
                        break;
                    }
                    OwnedMessage::Binary(bin) => {
                        let decoded = decoder.decode_float(&bin, false).unwrap();
                        producer.push_slice(&decoded);
                        // break; // debug!
                    }
                    _ => (),
                }
            }
            println!("Connection has been terminated.\n");
        });
    }
}
