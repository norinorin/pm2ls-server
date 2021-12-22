extern crate cpal;

use crate::decoder::OpusDecoder;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rb::{RbConsumer, RbProducer, SpscRb, RB};
use std::io;
use tokio::net::UdpSocket;

pub struct Player {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl Player {
    pub fn from_socket(socket: UdpSocket) -> Self {
        Self {
            socket,
            buf: vec![0; 1024],
        }
    }

    pub async fn run(self) -> Result<(), io::Error> {
        let Self { socket, mut buf } = self;

        let device = cpal::default_host()
            .default_output_device()
            .expect("Failed to get the output device.");

        // TODO: fetch it from the data header
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(48000),
            buffer_size: cpal::BufferSize::Fixed(960),
        };

        // max frame size * max frame rate / 1000 * max channels
        let ring = SpscRb::<f32>::new(5760);
        let (producer, consumer) = (ring.producer(), ring.consumer());

        let data_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let read = consumer.read(data).unwrap_or(0);
            data[read..].iter_mut().for_each(|s| *s = 0.0);
        };
        let output_stream = device
            .build_output_stream(&config, data_callback, Self::error_callback)
            .expect("Failed to create audio stream.");
        output_stream.play().unwrap();

        let decoder = OpusDecoder::new(48000, 1).unwrap();
        let mut to_send: Option<(usize, _)> = None;

        loop {
            if let Some((size, _)) = to_send {
                // TODO: have the audio info embedded in the header.
                let tmp = &buf[..size];
                trace!("Received: {:?}", tmp);
                let decoded = decoder.decode_float(tmp, false).unwrap();
                trace!("Decoded: {:?}", decoded);
                let mut decoded = decoded.as_slice();
                while let Some(written) = producer.write_blocking(decoded) {
                    decoded = &decoded[written..];
                }
            }

            to_send = Some(socket.recv_from(&mut buf).await?);
        }
    }

    fn error_callback(error: cpal::StreamError) {
        error!("Stream error: {}", error);
    }
}
