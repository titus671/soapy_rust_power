use crate::config;
use crate::sql_tools;
use num_complex::Complex;
use soapysdr::Device;
use soapysdr::Direction;
use std::f32::consts::PI;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Process SDR signal and extract RSSI for multiple frequencies
pub fn get_signal(config: &config::Config, tx: mpsc::Sender<sql_tools::TSData>) {
    let dev = Device::new(()).expect("Couldn't open device");

    // Configure the SDR
    dev.set_frequency(Direction::Rx, 0, config.sdr.center_frequency, ())
        .expect("Couldn't set frequency");
    dev.set_sample_rate(Direction::Rx, 0, config.sdr.sample_rate)
        .expect("Error setting sample rate");
    dev.set_gain(Direction::Rx, 0, config.sdr.gain)
        .expect("Error setting gain");

    // Set up streaming
    let mut stream = dev
        .rx_stream::<Complex<f32>>(&[0])
        .expect("Error getting stream");
    stream.activate(None).expect("Error activating stream");

    let mut buffer = vec![Complex::new(0.0, 0.0); 1024];

    // Define target frequencies within the SDR bandwidth
    //let target_freqs = config.sdr.frequencies;

    let sample_rate = config.sdr.sample_rate as f32;

    loop {
        buffer.fill(Complex::new(0.0, 0.0));
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        match stream.read(&mut [&mut buffer], 5_000_000) {
            Ok(samples) => {
                let mut power_values = vec![];

                for &freq in &config.sdr.frequencies {
                    let extracted_signal = downconvert_and_filter(
                        &buffer,
                        freq,
                        config.sdr.center_frequency,
                        sample_rate,
                    );

                    // Compute power
                    let power: f32 = extracted_signal.iter().map(|iq| iq.norm_sqr()).sum::<f32>()
                        / samples as f32;
                    let rssi = 10.0 * power.log10();
                    power_values.push((freq, rssi));
                }

                // Print results
                for (freq, rssi) in power_values {
                    //println!("Freq: {:.3} MHz, RSSI: {:.2} dB", freq as f32 / 1e6, rssi);
                    let frequency = freq / 1e6;
                    let data =
                        sql_tools::TSData::new(unix_time, config.id.unwrap(), frequency, rssi);
                    tx.send(data).expect("Failed to send data from thread");
                }
            }
            Err(e) => eprintln!("Error reading stream: {:?}", e),
        }
    }
}

/// Downconvert and filter the signal for a specific frequency
fn downconvert_and_filter(
    samples: &[Complex<f32>],
    target_freq: f64,
    center_freq: f64,
    sample_rate: f32,
) -> Vec<Complex<f32>> {
    let n = samples.len();
    let mut downconverted = vec![Complex::new(0.0, 0.0); n];

    // Compute frequency shift factor
    let freq_shift = (target_freq - center_freq) as f32 / sample_rate;

    for i in 0..n {
        let phase = -2.0 * PI * freq_shift * i as f32;
        let mixer = Complex::new(phase.cos(), phase.sin());
        downconverted[i] = samples[i] * mixer;
    }

    // Apply low-pass filter (basic moving average)
    let mut filtered = vec![Complex::new(0.0, 0.0); n];
    let filter_size = 20; // Simple averaging filter
    for i in filter_size..n {
        filtered[i] = downconverted[i - filter_size..i]
            .iter()
            .sum::<Complex<f32>>()
            / (filter_size as f32);
    }

    filtered
}
