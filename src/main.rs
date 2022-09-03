// Wav format see http://soundfile.sapp.org/doc/WaveFormat/

use std::fs::File;
use std::path::Path;
use std::f32::consts::PI;
//use std::io::prelude::*;

use wav::{Header};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a sine wave
    Sine {
        /// Frequency of the sine wave in hertz (default is 432 hertz)
        #[clap(short, long, action)]
        frequency: Option<u32>,

        /// Duration of the generated sine wave in seconds (default is 5 seconds) 
        #[clap(short, long, action)]
        duration: Option<u32>,

        /// Volume of the generated sine wave from 0 to 65 535 (default is 1000)
        #[clap(short, long, action)]
        volume: Option<u16>,
        
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
        #[clap(short, long, action)]
        start: u32,

        /// The finishing freqency in hertz
        #[clap(short, long, action)]
        finish: u32,

        /// Duration of the generated wave in seconds
        #[clap(short, long, action)]
        duration: Option<u32>,

        /// Volume of the generated sine wave from 0 to 65 535 (default is 1000)
        #[clap(short, long, action)]
        volume: Option<u16>,
    }
}

fn main() {

    let cli = Cli::parse();

    let sampling_rate = 44100;  // DEFAULT

    match cli.command {
        Commands::Sine { frequency, duration, volume, out_file_name } => {
            let mut data = Vec::<i16>::new();
            sine_wave(&mut data, frequency.unwrap_or(432), duration.unwrap_or(5), volume.unwrap_or(2000), sampling_rate);
            let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);
            let out_path = Path::new(&out_file_name);
            let mut out_file = File::create(out_path).expect("Unable to create the wav file ");
            wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file).expect("Unable to write to wav file");
            println!("Finished writing to {}", out_path.display());
        },
        Commands::Sweep {out_file_name, start, finish, duration, volume} => {
            let mut data = Vec::<i16>::new();

            sweep_wave(&mut data, start, finish, duration.unwrap_or(5), volume.unwrap_or(2000), sampling_rate);

            let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);
            let out_path = Path::new(&out_file_name);
            let mut out_file = File::create(out_path).expect("Unable to create the wav file ");
            wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file).expect("Unable to write to wav file");
            println!("Finished writing to {}", out_path.display());
        },
        
    }

    
}

fn sine_wave(data: &mut Vec<i16>, frequency: u32, duration: u32, volume: u16, sampling_rate: u32) {

    let n_samples = sampling_rate * duration;
    for t in 0..n_samples {
    // n_samples / duration = number sample for a second = s_samples

    // P_samples = s_sample / freqency  = samples in one period 

    // radians = t * 2pi / p_samples

    // radions = t * 2pi * f * duration / n_samples

        let r = (t as f32 * 2. * PI * frequency as f32 * duration as f32) / n_samples as f32;
        let amplitude = (r.sin() * volume as f32) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude );
        data.push(amplitude );
        
    }

}

fn sweep_wave(data: &mut Vec<i16>, start: u32, finish: u32, duration: u32, volume: u16, sampling_rate: u32) {
    let n_samples = sampling_rate * duration;

    let frequency_increment: f32 = (finish as f32 - start as f32) / n_samples as f32;
    let mut sweep_frequency: f32 = start as f32;

    for t in 0..n_samples {
        let r = (t as f32 * 2. * PI * sweep_frequency * duration as f32) / n_samples as f32;
        let amplitude = (r.sin() * volume as f32) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude );
        data.push(amplitude );

        // Adjust the fruquency for the next iteration
        sweep_frequency = sweep_frequency + frequency_increment;
    }

    
}
            
