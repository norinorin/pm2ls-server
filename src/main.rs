extern crate cpal;
extern crate pretty_env_logger;
extern crate websocket;
#[macro_use]
extern crate log;

mod decoder;

use cpal::platform::Device;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use decoder::OpusDecoder;
use rb::{RbConsumer, RbProducer, SpscRb, RB};
use std::fmt;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use websocket::sync::Client;
use websocket::sync::Server;
use websocket::ws::dataframe::DataFrame;
use websocket::OwnedMessage;

const LOG_LEVEL_VAR: &'static str = "LOG_LEVEL";

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
    if std::env::var(LOG_LEVEL_VAR).is_err() {
        std::env::set_var(LOG_LEVEL_VAR, "INFO");
    }

    pretty_env_logger::init_custom_env(LOG_LEVEL_VAR);

    let server = Server::bind("0.0.0.0:7619").unwrap();
    info!("Listening on port {}", 7619);

    let data = Arc::new(Mutex::new(DeviceWrapper {
        output_device: cpal::default_host().default_output_device().unwrap(),
    }));

    for request in server.filter_map(Result::ok) {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            info!("Got a connection! Trying to accept...");

            let data_ref = match data.try_lock() {
                Ok(data_ref) => data_ref,
                _ => {
                    error!("Currently in use. Rejecting incoming request.",);
                    request.reject().unwrap();
                    return;
                }
            };

            let mut client = request.accept().unwrap();

            info!("Connection has been established.");

            let sample_rate: u32 = get_int(&mut client);
            let channels: u16 = get_int(&mut client);
            let buffer_size: u32 = get_int(&mut client);
            let ring_buffer_size = buffer_size as usize * channels as usize;
            let latency = buffer_size / (sample_rate / 1000);

            info!("Trying to make a stream with:");
            info!("sample rate: {}", sample_rate);
            info!("channels: {}", channels);
            info!("buffer size ({} msec latency): {}", latency, buffer_size);

            let config = cpal::StreamConfig {
                channels,
                sample_rate: cpal::SampleRate(sample_rate),
                buffer_size: cpal::BufferSize::Fixed(buffer_size),
            };
            // TODO: reuse decoder
            let decoder = OpusDecoder::new(sample_rate as i32, channels as i32).unwrap();
            let ring = SpscRb::new(ring_buffer_size * 2);
            let (producer, consumer) = (ring.producer(), ring.consumer());

            let output_stream = data_ref
                .output_device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let read = consumer.read(data).unwrap_or(0);
                        data[read..].iter_mut().for_each(|s| *s = 0.0);
                    },
                    move |error: cpal::StreamError| error!("Stream threw an error: {}", error),
                )
                .unwrap();

            output_stream.play().unwrap();

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
                        let mut decoded = decoded.as_slice();
                        while let Some(written) = producer.write_blocking(decoded) {
                            decoded = &decoded[written..];
                        }
                        // break; // debug!
                    }
                    _ => (),
                }
            }
            info!("Connection has been terminated.\n");
        });
    }
}
