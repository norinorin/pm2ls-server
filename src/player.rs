extern crate cpal;

use crate::decoder::OpusDecoder;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rb::{RbConsumer, RbProducer, SpscRb, RB};
use tokio::net::UdpSocket;

pub struct Player {
    socket: UdpSocket,
    buf: Vec<u8>,
    write_all: bool,
    volume: i16,
}

impl Player {
    pub fn from_socket(socket: UdpSocket, write_all: bool, volume: i16) -> Self {
        Self {
            socket,
            buf: vec![0; 1024],
            write_all,
            volume,
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let Self {
            socket,
            mut buf,
            write_all,
            volume,
        } = self;

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
        output_stream.play()?;

        let decoder = OpusDecoder::new(48000, 1)?;
        decoder.set_volume(volume)?;

        let mut to_send: Option<(usize, _)> = None;

        let write_data = if write_all {
            info!("Running in blocking mode, will attempt to retry if writing fails.");
            warn!("The latency may be relatively high.");
            Self::write_all::<f32>
        } else {
            info!("Running in Low latency mode, will not attempt to retry if writing fails.");
            warn!("You might run into some crackles.");
            Self::try_write::<f32>
        };

        loop {
            if let Some((size, _)) = to_send {
                // TODO: have the audio info embedded in the header
                // and have a protocol to let the client know that we're connected.
                let tmp = &buf[..size];
                match decoder.decode_float(tmp, false) {
                    Ok(decoded) => {
                        trace!("Received: {:?}", tmp);
                        trace!("Decoded: {:?}", decoded);
                        write_data(&producer, decoded)
                    }
                    Err(error) => error!("Failed to decode due to: {}", error),
                }
            }

            to_send = Some(socket.recv_from(&mut buf).await?);
        }
    }

    fn error_callback(error: cpal::StreamError) {
        error!("Stream error: {}", error);
    }

    fn write_all<T>(producer: &dyn RbProducer<T>, data: Vec<T>) {
        let mut data = data.as_slice();

        while let Some(written) = producer.write_blocking(data) {
            data = &data[written..];
        }
    }

    fn try_write<T>(producer: &dyn RbProducer<T>, data: Vec<T>) {
        producer.write(&data).unwrap_or(0);
    }
}
