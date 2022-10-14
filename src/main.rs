//! A utility program to generate wave forms as either a `wav` file or a rust array initialisation.
//!
//! The later can be used for embedded projects where a waveform needs to be generated.
//!
//!
//! # Example Usage
//!
//! ## Sine wave
//!
//! To generate a **sine wave** with:
//! - a frequency of 643 hertz
//! - a duration of 3 seconds
//! - an output wav file `sine.wav`
//!  
//!
//! ```console
//!  wav-gen wav sine --frequency 643 --duration 3 sine.wav
//! ```
//! Note: it is assumed throughout that a `wav-gen` alias has been crated for the executable `wav-gen.exe`
//!
//! ## Sweeping Sine Wave
//!
//! To generate a sine wave that:
//! - **sweeps** between the frequencies of 300 hertz and 1000 hertz
//! - has a duration of 5 seconds
//! - is in the wav file `sweep.wav`
//!
//! ```console
//! wav-gen wav sweep --start 300  --finish 1000 --duration 5 sweep.wav
//! ```
//! ## Harmonics
//!
//! To generate a wave that has a set of harmonics first define the harmonics using a csv file (for example `harmonics.csv`):
//!
//! ```text
//! frequency,amplitude
//! 500.0 , 0.3
//! 700.0 , 0.2
//! 750.0 , 0.1
//! ```
//! This specifies a wave with harmonics at 500Hz, 700Hz and 750Hz with respective amplitudes 0.3, 0.2 and 0.1.
//! The amplitudes will be normalised.
//!
//! Then use:
//!
//! ```console
//! wav-gen wav harmonics --infile harmonics.csv output_wave_file.wav
//! ```
//! ## Rust Data Arrays
//!
//! To generate a sine waveform of 500Hz as a rust data array of 44140 words use the following
//!
//! ```console
//! wav-gen rust sine --frequency 500 --length 44100  ./wave.rs
//! ```
//!
//! The generated rust source code file looks like:
//!
//! ```
//! pub static DATA: [i16; 1024] = [
//!    // i16 values
//! ];
//! ```
//! The i16 values alternate between the left channel first and then the right channel. For stereo, each channel has the same value.
//!
//! If the structure has been, for instance, generated in the file `wave.rs` then it can be imported with:
//!
//! ```
//! use wave;
//! ```
//!
//!
//! Alternatively monophonic data arrays can be generated:
//!
//! ```console
//! wav-gen rust sine --frequency 500 --length 1024 --mono ./wave_mono-rs
//! ```
//!
//! This works the same for the other wave types such as `sweep` and `harmonics`.
//!
//! A different name for the rust data structure can be specified:
//! ```console
//! wav-gen rust sweep --start 500 --finish 1500 --name SWEEP_DATA ./sweep.rs
//! ```
//!
//! The generated rust source code looks like:
//!
//! ```
//! pub static SWEEP_DATA: [i16; 1024] = [
//!          0,     0,    71,    71,   143,   143,   214,   214,   285,   285,
//!        355,   355,   423,   423,   490,   490,   554,   554,   616,   616,
//!        // ... more i16 values ...
//!        947,   947,   777,   777,
//! ];
//! ```
//!
//! For sine waves and harmonics, instead of generating a rust source code file with a large number of samples, only one cycle can be generated
//! by using the `--cycle` flag, e.g.:
//!  
//!
//! ```console
//! wav-gen rust sine --frequency 2000 --cycle  ./src/SINE_DATA.rs
//! ```
//!
//! # More options
//! For more options use:
//!
//! ```console
//! wav-gen help
//! ```

//  Wav format specification: see http://soundfile.sapp.org/doc/WaveFormat/

use num::integer::lcm;
use std::error::Error;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use wav::Header;

use clap::{Args, CommandFactory, ErrorKind, Parser, Subcommand, ValueEnum};

mod error;

type WavGenError = error::WavGenError;

/// Structure used by the `clap` to process the command line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)] // Read from `Cargo.toml`

struct Cli {
    /// Name of the output wave file
    #[clap(global = true, default_value_t = String::from("sine.wav"), value_parser)]
    out_file_name: String,

    /// Volume of the generated wave from 0 to 65 535
    #[clap(global = true, short, long, value_parser, default_value = "1000")]
    volume: u16,

    #[clap(subcommand)]
    command: OutputTypeCommands,
}

#[derive(Subcommand)]
enum OutputTypeCommands {
    /// Generate a wav file
    Wav(WavOptions),
    /// Generate a rust data structure
    Rust(RustOptions),
}

#[derive(Args)]
struct WavOptions {
    /// Duration of the generated sine wave in seconds.
    #[clap(global = true, short, long, value_parser, default_value = "5")]
    duration: u32,

    #[clap(subcommand)]
    gen_command: GenCommands,
}

#[derive(Args)]
struct RustOptions {
    /// Length of the generated wave in words. Independent of stereo or mono, only
    /// this number of entries will be generated. For stereo the length needs to be
    /// an even number
    #[clap(global = true, short, long, value_parser, default_value = "1024")]
    length: u32,

    /// Generate just one cycle of the waveform. Cannot be used with --length
    #[clap(global = true, short, long, action, conflicts_with("length"))]
    cycle: bool,

    /// Name of the rust data struct generated
    #[clap(global = true, short, long, value_parser, default_value = "DATA")]
    name: String,

    /// Only samples for one channel are generated. This is different
    /// to stereo (the default) where an entry for the left and then
    /// for the right channel is generated
    #[clap(global = true, short, long, action, default_value_t = false)]
    mono: bool,

    #[clap(subcommand)]
    gen_command: GenCommands,
}

/// Structure used by the `clap` to process the subcommands
#[derive(Subcommand)]
enum GenCommands {
    /// Generate a sine wave
    Sine {
        /// Frequency of the sine wave in hertz
        #[clap(short, long, value_parser, default_value = "432")]
        frequency: u32,
    },

    /// Generate a sine wave that sweeps from one frequency to another over the duration
    Sweep {
        /// The starting freqency in hertz
        #[clap(short, long, value_parser, default_value = "100")]
        start: u32,

        /// The finishing freqency in hertz
        #[clap(short, long, value_parser, default_value = "2000")]
        finish: u32,
    },

    /// Generate a wave that combines the sine waves specified in a external csv file.
    Harmonics {
        /// Name of the csv file containing the harmonics
        #[clap(short, long, default_value_t = String::from("harmonics.csv"),value_parser)]
        infile: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputType {
    Wav,
    Rust,
}

enum GeneratedSize {
    NumberSamples(u32),
    Cyclic,
}

/// Represents an harmonic as a frequency and it's relative amplitude to other harmonics
#[allow(dead_code)]
#[derive(Debug)]
struct Harmonic {
    frequency: u32, // In hertz
    amplitude: f32,
}

/// Generate wav files from the command line arguments provided.
fn main() -> Result<(), WavGenError> {
    let cli = Cli::parse();

    let sampling_rate = 44100; // DEFAULT
                               //let number_channels = 2; // DEFAULT

    // Process output type command options
    let (size, number_channels) = match cli.command {
        OutputTypeCommands::Wav(ref wav_options) => (
            GeneratedSize::NumberSamples(wav_options.duration * sampling_rate),
            2,
        ),
        OutputTypeCommands::Rust(ref rust_options) => {
            // If stereo need a even length so that left and right samples are present
            if !rust_options.mono && rust_options.length % 2 != 0 {
                let mut cmd = Cli::command();
                cmd.error(
                    ErrorKind::InvalidValue,
                    "With stereo the length of the data structure needs to be an even number",
                )
                .exit();
            }
            let n_channels: u8 = if rust_options.mono { 1 } else { 2 };
            let size = if !rust_options.cycle {
                GeneratedSize::NumberSamples(rust_options.length / n_channels as u32)
            } else {
                GeneratedSize::Cyclic
            };

            (size, n_channels)
        }
    };

    let gen_command = match cli.command {
        OutputTypeCommands::Wav(ref wav_options) => &wav_options.gen_command,
        OutputTypeCommands::Rust(ref rust_options) => &rust_options.gen_command,
    };

    let data = match gen_command {
        GenCommands::Sine { frequency } => {
            let n_samples = match size {
                GeneratedSize::Cyclic => sampling_rate * number_channels as u32 / frequency,
                GeneratedSize::NumberSamples(number_samples) => number_samples,
            };
            gen_sine_wave(
                *frequency,
                n_samples,
                number_channels,
                cli.volume,
                sampling_rate,
            )
        }
        GenCommands::Sweep { start, finish } => {
            let n_samples = match size {
                GeneratedSize::Cyclic => {
                    let mut cmd = Cli::command();
                    cmd.error(
                        ErrorKind::ArgumentConflict,
                        "Specifying --cycle for the subcommand sweep is not meaningful",
                    )
                    .exit();
                }
                GeneratedSize::NumberSamples(n_samples) => n_samples,
            };

            gen_sweep_wave(
                *start,
                *finish,
                n_samples,
                number_channels,
                cli.volume,
                sampling_rate,
            )
        }

        GenCommands::Harmonics { infile } => {
            let p = Path::new(infile);
            let mut harmonics_set =
                read_harmonics(p).map_err(|_| WavGenError::ReadError(p.to_path_buf()))?;
            normalise_harmonics(&mut harmonics_set);

            let n_samples = match size {
                GeneratedSize::Cyclic => {
                    let frequencies = harmonics_set.iter().map(|h| h.frequency).collect();
                    sync_period(&frequencies, sampling_rate)
                }
                GeneratedSize::NumberSamples(n_samples) => n_samples,
            };

            gen_harmonics(
                &harmonics_set,
                n_samples,
                number_channels,
                cli.volume,
                sampling_rate,
            )?
        }
    };

    let out_path = Path::new(&cli.out_file_name);
    let mut out_file =
        File::create(out_path).map_err(|_| WavGenError::CreateError(out_path.to_path_buf()))?;

    match cli.command {
        OutputTypeCommands::Wav(_) => {
            let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);
            wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file)
                .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
        }
        OutputTypeCommands::Rust(rust_options) => {
            write_rust(&data, rust_options.name.as_str(), out_path, &mut out_file)?;
        }
    };

    bunt::println!(
        "{$bold+green}Finished{/$} writing to {}",
        out_path.display()
    );

    Ok(())
}

/// Generate a sine wave as a set of `i16` samples and returns this.
///
/// # Arguments
/// * `frequency`- The frequency of the sine wave in hertz
/// * `number_samples` - the number of samples to be generated.
///                      The duration of the generated wave is the `number_samples/sampling_rate`.
/// * `number_channels` - The number of channels (1 or 2)
/// * `volume`- The volume of the generated sine wave
/// * `sampling_rate`- The rate at which the wave wave is sampled, e.g 44100 hertz.
///                    The `sample_rate` and the `duration` determine the the size of `data`  
fn gen_sine_wave(
    frequency: u32,
    number_samples: u32,
    number_channels: u8,
    volume: u16,
    sampling_rate: u32,
) -> Vec<i16> {
    let mut data = Vec::<i16>::new();

    for t in 0..number_samples {
        let radians = (t as f32 * 2. * PI * frequency as f32) / sampling_rate as f32;
        let amplitude = (radians.sin() * volume as f32) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude);
        if number_channels == 2 {
            data.push(amplitude);
        }
    }

    data
}

/// Generate a sweeping sine wave as a set of `i16` samples and returns it
///
/// # Arguments
/// * `start` - The start frequency of sweep in hertz
/// * `finish`- The finishing frequency of the sweep in hertz
/// * ´number_samples" - the number of samples to be generated.
///                      The duration of the generated wave is the `number_samples/sampling_rate`.
/// * `number_channels` - The number of channels (1 or 2)
/// * `volume`- The volume of the generated wave
/// * `sampling_rate`- The rate at which the wave wave is sampled, e.g 44100 hertz.
///                    The `sample_rate` and the `duration` determine the the size of `data`  
fn gen_sweep_wave(
    start: u32,
    finish: u32,
    number_samples: u32,
    number_channels: u8,
    volume: u16,
    sampling_rate: u32,
) -> Vec<i16> {
    let mut data = Vec::<i16>::new();

    let frequency_increment: f32 = (finish as f32 - start as f32) / number_samples as f32;
    let mut sweep_frequency: f32 = start as f32;

    for t in 0..number_samples {
        let r = (t as f32 * 2. * PI * sweep_frequency) / sampling_rate as f32;
        let amplitude = (r.sin() * volume as f32) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude);
        if number_channels == 2 {
            data.push(amplitude);
        }

        // Adjust the frequency for the next iteration
        sweep_frequency += frequency_increment;
    }

    data
}

#[allow(unused_variables)]
fn gen_harmonics(
    harmonics_set: &[Harmonic],
    number_samples: u32,
    number_channels: u8,
    volume: u16,
    sampling_rate: u32,
) -> Result<Vec<i16>, WavGenError> {
    // Generate a initial set of data
    if let Some(h) = harmonics_set.first() {
        let mut data = gen_sine_wave(
            h.frequency,
            number_samples,
            number_channels,
            (h.amplitude * volume as f32) as u16,
            sampling_rate,
        );
        // Overlay the other harmonics
        for harmonic_entry in harmonics_set.iter().skip(1) {
            let overlay_data = gen_sine_wave(
                harmonic_entry.frequency as u32,
                number_samples,
                number_channels,
                (harmonic_entry.amplitude * volume as f32) as u16,
                sampling_rate,
            );

            for i in 0..data.len() {
                data[i] += overlay_data[i];
            }
        }
         Ok(data)
    } else {
        Err(WavGenError::NoHarmonics)
    }
}

fn read_harmonics(harmonics_path: &Path) -> Result<Vec<Harmonic>, Box<dyn Error>> {
    //fn read_harmonics(harmonics_path: &Path) -> Result<Vec<Harmonic>,  HarmonicReadError> {

    let mut rdr = csv::Reader::from_path(harmonics_path)?;
    let mut harmonics = Vec::<Harmonic>::new();

    let mut line_number = 1;

    for result in rdr.records() {
        let record = result.map_err(|_| WavGenError::ReadError(harmonics_path.to_path_buf()))?;

        let f: u32 = record
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

/// Normalise the amplitudes of the harmonics so that the sum of them all is 1
fn normalise_harmonics(harmonics_set: &mut Vec<Harmonic>) {
    let mut sum = 0.;
    for h in harmonics_set.iter_mut() {
        sum += h.amplitude;
    }

    for h in harmonics_set.iter_mut() {
        h.amplitude /= sum;
    }
}

fn write_rust(
    data: &Vec<i16>,
    data_struct_name: &str,
    out_path: &Path,
    out_file: &mut File,
) -> Result<(), WavGenError> {
    let mut buf_writer = BufWriter::new(out_file);

    writeln!(
        buf_writer,
        "pub static {}: [i16; {}] = [",
        data_struct_name,
        data.len()
    )
    .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;

    let mut block_count = 0;
    for sample in data {
        if block_count == 0 {
            write!(buf_writer, "    ")
                .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
        }
        write!(buf_writer, "{:6},", sample)
            .map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
        block_count += 1;
        if block_count == 10 {
            writeln!(buf_writer).map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
            block_count = 0;
        }
    }

    writeln!(buf_writer).map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;
    writeln!(buf_writer, "];").map_err(|_| WavGenError::WriteError(out_path.to_path_buf()))?;

    Ok(())
}

/// Finds the least common numerator of the periods in a set of sine waves, i.e the time (in number of samples) at which
/// all the sine wave start at zero (are sychronised) again.
#[allow(dead_code)]
fn sync_period(frequencies: &Vec<u32>, sampling_rate: u32) -> u32 {
    let scale: u32 = 20000; // Used to scale up each period so that it remains an integer
    let scaled_sample_periods: Vec<u32> = frequencies
        .iter()
        .map(|f| sampling_rate * scale / f)
        .collect();

    let mut period: u32 = 1;
    for v in scaled_sample_periods.iter() {
        period = lcm(period, *v);
    }

    period / scale
}
