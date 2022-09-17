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
//! ## Rust data arrays
//!
//! To generate a sine waveform of 500Hz as a rust data array of 44140 words use the following
//!
//! ```console
//! wav-gen rust sine --frequency 500 --length 44100  wave.rs
//! ```
//!
//! The generated rust source code file looks like:
//!
//! ```
//! static DATA: [i16; 1024] = [
//!    // i16 values
//! ];
//! ```
//! The i16 fvalues alternate between the left channel first and then the right channel. As stereo is not supported
//! as yet, each channel has the same value.
//!
//! This works the same for the other wave types such as `sweep` and `harmonics`.
//!
//! Notes:
//! * The name of the data array is a capitalized version of the file name.
//! * For the rust output type any duration specified is an error.
//!
//! # More options
//! For more options use:
//!
//! ```console
//! wav-gen help
//! ```

//  Wav format see http://soundfile.sapp.org/doc/WaveFormat/

//use core::num;
use bunt;
use std::error::Error;
use std::f32::consts::PI;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use wav::Header;

use clap::{Args, CommandFactory, ErrorKind, Parser, Subcommand, ValueEnum};

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
    Wav(WavOptions),
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
    /// this number of entries wil be generated. For stereo the length needs to be 
    /// an even number
    #[clap(global = true, short, long, value_parser, default_value = "1024")]
    length: u32,

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

    /// Generate a wave that sweeps from one frequency to another over the duration
    Sweep {
        /// The starting freqency in hertz
        #[clap(short, long, value_parser, default_value = "100")]
        start: u32,

        /// The finishing freqency in hertz
        #[clap(short, long, value_parser, default_value = "2000")]
        finish: u32,
    },

    /// Generate a wave that contains the harmonics specified in a external csv file.
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
                               //let number_channels = 2; // DEFAULT

    let (number_samples, number_channels) = match cli.command {
        OutputTypeCommands::Wav(ref wav_options) => (wav_options.duration * sampling_rate, 2),
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
            let n_samples = rust_options.length / n_channels as u32;
            (n_samples, n_channels)
        }
    };

    let gen_command = match cli.command {
        OutputTypeCommands::Wav(ref wav_options) => &wav_options.gen_command,
        OutputTypeCommands::Rust(ref rust_options) => &rust_options.gen_command,
    };

    let data = match gen_command {
        GenCommands::Sine { frequency } => gen_sine_wave(
            *frequency,
            number_samples,
            number_channels,
            cli.volume,
            sampling_rate,
        ),
        GenCommands::Sweep { start, finish } => gen_sweep_wave(
            *start,
            *finish,
            number_samples,
            number_channels,
            cli.volume,
            sampling_rate,
        ),
        GenCommands::Harmonics { infile } => {
            let p = Path::new(infile);
            let mut harmonics_set =
                read_harmonics(p).map_err(|_| WavGenError::ReadError(p.to_path_buf()))?;
            normalise_harmonics(&mut harmonics_set);
            gen_harmonics(
                &harmonics_set,
                number_samples,
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
        sweep_frequency = sweep_frequency + frequency_increment;
    }

    data
}

#[allow(unused_variables)]
fn gen_harmonics(
    harmonics_set: &Vec<Harmonic>,
    number_samples: u32,
    number_channels: u8,
    volume: u16,
    sampling_rate: u32,
) -> Result<Vec<i16>, WavGenError> {
    // Generate a initial set of data
    if let Some(h) = harmonics_set.first() {
        let mut data = gen_sine_wave(
            h.frequency as u32,
            number_samples,
            number_channels,
            (h.amplitude * volume as f32) as u16,
            sampling_rate,
        );
        // Overlay the other harmonics
        for harmonic_index in 1..harmonics_set.len() {
            let overlay_data = gen_sine_wave(
                harmonics_set[harmonic_index].frequency as u32,
                number_samples,
                number_channels,
                (harmonics_set[harmonic_index].amplitude * volume as f32) as u16,
                sampling_rate,
            );

            for i in 0..data.len() {
                data[i] = data[i] + overlay_data[i];
            }
        }
        return Ok(data);
    } else {
        return Err(WavGenError::NoHarmonics);
    }
}

// Error handling
// TODO move to another file

enum WavGenError {
    ReadError(PathBuf),
    WriteError(PathBuf),
    CreateError(PathBuf),
    HarmonicParseError(usize),
    NoHarmonics,
}

//Required for the ? operator
impl std::error::Error for WavGenError {}

// Display for the error
impl fmt::Display for WavGenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WavGenError::ReadError(p) => f.write_fmt(format_args!("could not read file {:?}", p)),
            WavGenError::WriteError(p) => f.write_fmt(format_args!("could not write file {:?}", p)),
            WavGenError::CreateError(p) => {
                f.write_fmt(format_args!("unable to create file {:?}", p))
            }
            WavGenError::HarmonicParseError(line_number) => f.write_fmt(format_args!(
                "parse error in harmonic file at line {:?}",
                line_number
            )),
            WavGenError::NoHarmonics => f.write_fmt(format_args!("no harmonics found")),
        }
    }
}

// Using the display implmentation for the debug implementation means that
// user friendly messages are shown when the main funtion exists with
// an error
impl std::fmt::Debug for WavGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <WavGenError as fmt::Display>::fmt(self, f)
    }
}

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

/// Normalise the amplitudes of the harmonics so that the sum of them all is 1
fn normalise_harmonics(harmonics_set: &mut Vec<Harmonic>) {
    let mut sum = 0.;
    for h in harmonics_set.iter_mut() {
        sum += h.amplitude;
    }

    for h in harmonics_set.iter_mut() {
        h.amplitude = h.amplitude / sum;
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
        "static {}: [i16; {}] = [",
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
