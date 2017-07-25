use std::env;
extern crate soapysdr;
extern crate num_complex;
use soapysdr::Direction::{Rx, Tx};
use std::fmt;

fn main() {
    let filter = env::args().nth(1).unwrap_or(String::new());

    for (i, devargs) in soapysdr::enumerate(&filter[..]).expect("Error listing devices").iter().enumerate() {
        println!("Device #{}: {}", i, devargs);

        let mut dev = soapysdr::Device::new(devargs).expect("Failed to open device");

        let hardware_info = dev.hardware_info().unwrap();
        println!("Hardware info: {}", hardware_info);

        for channel in 0..(dev.num_channels(Rx).unwrap_or(0)) {
            print_channel_info(&dev, Rx, channel).expect("Failed to get channel info");
        }

        for channel in 0..(dev.num_channels(Tx).unwrap_or(0)) {
            print_channel_info(&dev, Tx, channel).expect("Failed to get channel info");
        }

        let stream_result = stream_data(&mut dev);
        println!("stream_result: {:?}", stream_result);
    }
}

fn stream_data(dev: &mut soapysdr::Device) -> Result<(), soapysdr::Error> {
    println!("Setting frequency");
    dev.set_frequency(Rx, 0, 433_000_000.0, &soapysdr::Args::new())?;

    println!("Creating Rx stream");
    let mut stream = dev.rx_stream::<num_complex::Complex<i16>>(&[0], &soapysdr::Args::new())?;

    println!("Activating Rx stream");
    stream.activate(None)?;
    let mut data_channel_0 = vec![Default::default(); 1000];
    let data_channels = [data_channel_0.as_mut_slice()];

    println!("Reading Rx stream");
    let read_result = stream.read(&data_channels, 10_000_000_000/*ns timeout*/)?;
    println!("Successfully read {} of {} entries from stream", data_channels[0].len(), read_result);
    println!("{:?}", data_channels[0]);

    println!("Deactivating Rx stream");
    stream.deactivate(None)?;
    Ok(())
}

struct DisplayRange(Vec<soapysdr::Range>);

impl fmt::Display for DisplayRange {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        for (i, range) in self.0.iter().enumerate() {
            if i != 0 { write!(w, ", ")? }
            if range.minimum == range.maximum {
                write!(w, "{} MHz", range.maximum / 1e6)?
            } else {
                write!(w, "{} to {} MHz", range.minimum / 1e6, range.maximum / 1e6)?
            }
        }
        Ok(())
    }
}

fn print_channel_info(dev: &soapysdr::Device, dir: soapysdr::Direction, channel: usize) -> Result<(), soapysdr::Error> {
    let dir_s = match dir { Rx => "Rx", Tx => "Tx"};
    println!("\t{} Channel {}", dir_s, channel);

    let freq_range = dev.frequency_range(dir, channel)?;
    println!("\t\tFreq range: {}", DisplayRange(freq_range));

    let sample_rates = dev.get_sample_rate_range(dir, channel)?;
    println!("\t\tSample rates: {}", DisplayRange(sample_rates));

    let (native, full_scale) = dev.native_stream_format(dir, channel)?;
    println!("\t\tNative format: {} {}", native, full_scale);

    println!("\t\tAvailable formats:");
    for format in dev.stream_formats(dir, channel)? {
        println!("\t\t\t{}", format);
    }

    let info = dev.channel_info(dir, channel)?;
    println!("\t\tChannel info: {}", info);

    println!("\t\tAntennas: ");
    for antenna in dev.antennas(dir, channel)? {
        println!("\t\t\t{}", antenna);
    }

    Ok(())
}
