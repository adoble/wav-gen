//! ## Introduction
//!
//! A utility program to generate wave forms as a `wav` file.
//!
//!
//! ## Example Usage
//!
//! To generate a **sine wave** with:
//! - a frequency of 643 hertz
//! - a duration of 3 seconds
//! - an output wav file `sine.wav`
//!  
//!
//! ```console
//!  wav-gen sine --frequency 643 --duration 3 sine.wav
//! ```
//! To generate a sine wave that:
//! - **sweeps** between the frequencies of 300 hertz and 1000 hertz
//! - has a duration of 5 seconds
//! - is in the wav file `sweep.wav`
//!
//! ```console
//! wav-gen sweep --start 300  --finish 1000 --duration 5 sweep.wav
//! ```
//!
//! For more options use:
//!
//! ```console
//! wav-gen help
//! ```

//  Wav format see http://soundfile.sapp.org/doc/WaveFormat/

use std::error::Error;
use std::f32::consts::PI;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
//use std::io::prelude::*;

use wav::Header;

use clap::{Parser, Subcommand};

/// Structure used by the `clap` to process the command line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

/// Structure used by the `clap` to process the subcommands
#[derive(Subcommand)]
enum Commands {
    /// Generate a sine wave
    Sine {
        /// Frequency of the sine wave in hertz
        #[clap(short, long, value_parser, default_value = "432")]
        frequency: u32,

        /// Duration of the generated sine wave in seconds
        #[clap(short, long, value_parser, default_value = "5")]
        duration: u32,

        /// Volume of the generated sine wave from 0 to 65 535
        #[clap(short, long, value_parser, default_value = "1000")]
        volume: u16,

        /// Name of the output wave file
        #[clap(default_value_t = String::from("sine.wav"), value_parser)]
        out_file_name: String,
    },
    /// Generate a wave that sweeps from one frequency to another over the duration
    Sweep {
        /// Name of the output wave file
        #[clap(default_value_t = String::from("sweep.wav"),value_parser)]
        out_file_name: String,

        /// The starting freqency in hertz
        #[clap(short, long, value_parser, default_value = "100")]
        start: u32,

        /// The finishing freqency in hertz
        #[clap(short, long, value_parser, default_value = "2000")]
        finish: u32,

        /// Duration of the generated wave in seconds
        #[clap(short, long, value_parser, default_value = "5")]
        duration: u32,

        /// Volume of the generated sine wave from 0 to 65 535
        #[clap(short, long, value_parser, default_value = "1000")]
        volume: u16,
    },
    /// Generate a wave that contains the harmonics specified in a external csv file.
    Harmonics {
        /// Name of the output wave file
        #[clap(default_value_t = String::from("harmonics.wav"),value_parser)]
        out_file_name: String,

        /// Name of the csv file containing the harmonics
        #[clap(short = 'm', long, default_value_t = String::from("harmonics.csv"),value_parser)]
        harmonics: String,

        /// Duration of the generated wave in seconds
        #[clap(short, long, value_parser, default_value = "5")]
        duration: u32,

        /// Volume of the generated sine wave from 0 to 65 535
        #[clap(short, long, value_parser, default_value = "1000")]
        volume: u16,
    },
}

/// Represents an harmonic as a frequency and it's relative amplitude to other harmonics
#[allow(dead_code)]
#[derive(Debug)]
struct Harmonic {
    frequency: f32, // In hertz
    amplitude: f32,
}

/// Generate wav files from the command line arguments provided.
fn main() -> Result<(), WavGenError> {
    let cli = Cli::parse();

    let sampling_rate = 44100; // DEFAULT

    match cli.command {
        Commands::Sine {
            frequency,
            duration,
            volume,
            out_file_name,
        } => {
            let data = gen_sine_wave(frequency, duration, volume, sampling_rate);
            let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);
            let out_path = Path::new(&out_file_name);
            let mut out_file = File::create(out_path).map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
            wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file)
                .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;

            println!("Finished writing to {}", out_path.display());
        }
        Commands::Sweep {
            out_file_name,
            start,
            finish,
            duration,
            volume,
        } => {
            let data = gen_sweep_wave(start, finish, duration, volume, sampling_rate);

            let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);
            let out_path = Path::new(&out_file_name);
            let mut out_file = File::create(out_path).expect("Unable to create the wav file ");
            wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file)
                .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
            println!("Finished writing to {}", out_path.display());
        }
        Commands::Harmonics {
            out_file_name,
            harmonics,
            duration,
            volume,
        } => {
            let p = Path::new(&harmonics);

            let mut harmonics_set = read_harmonics(p).expect("Error in reading harmonics file");

            normalise_harmonics(&mut harmonics_set);

            let mut data = Vec::<i16>::new();
            gen_harmonics(&mut data, &harmonics_set, duration, volume, sampling_rate);

            let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);
            let out_path = Path::new(&out_file_name);
            let mut out_file = File::create(out_path).expect("Unable to create the wav file ");
            wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file)
                .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
            println!("Finished writing to {}", out_path.display());
        }
    }
    Ok(())
}

/// Generate a sine wave as a set of `i16` samples an d returns this.
///
/// # Arguments
/// * `frequency`- The frequency of the sine wave in hertz
/// * `duration`- The duration of the generated waveform in seconds
/// * `volume`- The volume of the generated sine wave
/// * `sampling_rate`- The rate at which the wave wave is sampled, e.g 44100 hertz.
///                    The `sample_rate` and the `duration` determine the the size of `data`  
fn gen_sine_wave(
    frequency: u32,
    duration: u32,
    volume: u16,
    sampling_rate: u32,
) -> Vec<i16> {
    let n_samples = sampling_rate * duration;

    let mut data = Vec::<i16>::new();

    for t in 0..n_samples {
        // n_samples / duration = number sample for a second = s_samples

        // P_samples = s_sample / freqency  = samples in one period

        // radians = t * 2pi / p_samples

        // radions = t * 2pi * f * duration / n_samples

        let r = (t as f32 * 2. * PI * frequency as f32 * duration as f32) / n_samples as f32;
        let amplitude = (r.sin() * volume as f32) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude);
        data.push(amplitude);
    }

    data
}

/// Generate a sweeping sine wave as a set of `i16` samples and retuns it
///
/// # Arguments
/// * `start` - The start frequency of sweep in hertz
/// * `finish`- The finishing frequency of the sweep in hertz
/// * `duration`- The duration of the generated waveform in seconds
/// * `volume`- The volume of the generated wave
/// * `sampling_rate`- The rate at which the wave wave is sampled, e.g 44100 hertz.
///                    The `sample_rate` and the `duration` determine the the size of `data`  
fn gen_sweep_wave(
    start: u32,
    finish: u32,
    duration: u32,
    volume: u16,
    sampling_rate: u32,
) -> Vec<i16> {

    let mut data = Vec::<i16>::new();
    let n_samples = sampling_rate * duration;

    let frequency_increment: f32 = (finish as f32 - start as f32) / n_samples as f32;
    let mut sweep_frequency: f32 = start as f32;

    for t in 0..n_samples {
        let r = (t as f32 * 2. * PI * sweep_frequency * duration as f32) / n_samples as f32;
        let amplitude = (r.sin() * volume as f32) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude);
        data.push(amplitude);

        // Adjust the frequency for the next iteration
        sweep_frequency = sweep_frequency + frequency_increment;
    }

    data
}

#[allow(unused_variables)]
fn gen_harmonics(
    data: &mut Vec<i16>,
    harmonics_set: &Vec<Harmonic>,
    duration: u32,
    volume: u16,
    sampling_rate: u32,
) {
    println!("Not implemented!");
    println!(
        "Duration = {}, volume = {}, sample_rate = {}",
        duration, volume, sampling_rate
    );
    println!("Harmonics: {:?}", harmonics_set);

    todo!();

    // EXPERIMENTAL CODE - not finished
    // assert!(harmonics_set.len() > 0);

    // if let Some(h) = harmonics_set.first() {
    //     // Generate a intial set of data....
    // }

    // let mut h = harmonics_set.first().unwrap();
    // gen_sine_wave(&mut data, h.frequency as u32, duration, (h.amplitude * volume as f32) as u16, sampling_rate);

    // let mut overlay_data = Vec::<i16>::new();
    // for j in 1..harmonics_set.len() {

    //     gen_sine_wave(&mut overlay_data, harmonics_set[j].frequency as u32, duration, (harmonics_set[j].amplitude * volume as f32) as u16, sampling_rate);

    //     for i in 0..data.len() {
    //         data[i] = data[i] + overlay_data[i];
    //     }

    // }

    // OLD CODE
    // let mut data = Vec::<i16>::new();
    // let mut overlay_data = Vec::<i16>::new();

    // sine_wave(&mut data, frequency, duration, vol, sampling_rate);
    // sine_wave(&mut overlay_data, 800, duration, vol, sampling_rate);

    // // Add the two together with the result in data
    // for i in 0..data.len() {
    //     data[i] = data[i] + overlay_data[i];
    // }

    // sine_wave(&mut overlay_data, 1600, duration, 600, sampling_rate);
    // for i in 0..data.len() {
    //     data[i] = data[i] + overlay_data[i];
    // }
}

// Error handling
// TODO move to another file

#[derive(Debug)]
enum WavGenError {
    ReadError(PathBuf),
    WriteError(PathBuf),
    HarmonicParseError(usize),
}

//Required for the ? operator
impl std::error::Error for WavGenError {}

// Display for the error
impl fmt::Display for WavGenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WavGenError::ReadError(p) => f.write_fmt(format_args!("could not read file {:?}", p)),
            WavGenError::WriteError(p) => f.write_fmt(format_args!("could not write file {:?}", p)),
            WavGenError::HarmonicParseError(line_number) => f.write_fmt(format_args!(
                "parse error in harmonic file at line {:?}",
                line_number
            )),
        }
    }
}

// #[derive(Debug)]
// struct WavGenError {
//     line_number: usize,
//     path: PathBuf,
//     err: WavGenErrorType,
// }

// impl fmt::Display for WavGenError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self.err {
//             WavGenErrorType::ReadError => {
//                 f.write_fmt(format_args!("Could not read file {:?}", self.path))
//             }
//             WavGenErrorType::WriteError => {
//                 f.write_fmt(format_args!("Could not write file {:?}", self.path))
//             }
//             WavGenErrorType::HarmonicParseError => f.write_fmt(format_args!(
//                 "Parse error in harmonic file at line {:?}",
//                 self.line_number
//             )),
//         }
//     }
// }

// Required for the ? operator
//impl std::error::Error for WavGenError {}

fn read_harmonics(harmonics_path: &Path) -> Result<Vec<Harmonic>, Box<dyn Error>> {
    //fn read_harmonics(harmonics_path: &Path) -> Result<Vec<Harmonic>,  HarmonicReadError> {

    let mut rdr = csv::Reader::from_path(harmonics_path)?;
    let mut harmonics = Vec::<Harmonic>::new();

    let mut line_number = 1;

    for result in rdr.records() {
        let record = result.map_err(|_| WavGenError::ReadError(harmonics_path.to_path_buf()))?;

        let f: f32 = record
            .get(0)
            .ok_or_else(|| WavGenError::HarmonicParseError(line_number))?
            .trim()
            .parse()
            .map_err(|_| WavGenError::HarmonicParseError(line_number))?;

        let a: f32 = record
            .get(1)
            .ok_or_else(|| WavGenError::HarmonicParseError(line_number))?
            .trim()
            .parse()
            .map_err(|_| WavGenError::HarmonicParseError(line_number))?;

        harmonics.push(Harmonic {
            frequency: f,
            amplitude: a,
        });

        line_number += 1;
    }

    Ok(harmonics)
}

/// Normalise the amplitides of the harmonics so that the sum of them all is 1
fn normalise_harmonics(harmonics_set: &mut Vec<Harmonic>) {
    let mut sum = 0.;
    for h in harmonics_set.iter_mut() {
        sum += h.amplitude;
    }

    for h in harmonics_set.iter_mut() {
        h.amplitude = h.amplitude / sum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[test]
    fn test_read() {
        let path: &Path = Path::new("HARMONICS.csv");
        match read_harmonics(&path) {
            Ok(v) => v
                .iter()
                .for_each(|h| println!("{},{}", h.frequency, h.amplitude)),
            Err(e) => println!("Cannot read harmonics file: {}", e),
        }
    }

    #[test]
    fn test_read_harmonics() {
        let path: &Path = Path::new("../../sounds/harmonics.csv");
        match read_harmonics(&path) {
            Ok(v) => v
                .iter()
                .for_each(|h| println!("{},{}", h.frequency, h.amplitude)),
            Err(e) => println!("Cannot read harmonics file: {}", e),
        }
    }
}
