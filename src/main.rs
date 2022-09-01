// Wav format see http://soundfile.sapp.org/doc/WaveFormat/

use std::fs::File;
use std::f32::consts::PI;
//use std::io::prelude::*;

use wav::{Header};




fn main() {

    // Generate sine wave data for 5 secs
    let duration = 5; // 5 seconds
    let sampling_rate = 44100;
    let frequency = 500; // 500 Herz

    let mut data = Vec::<i16>::new();
    let n_samples = sampling_rate * duration;
    for t in 0..n_samples {
    // n_samples / duration = number sample for a second = s_samples

    // P_samples = s_sample / freqency  = samples in one period 

    // radians = t * 2pi / p_samples

    // radions = t * 2pi * f * duration / n_samples

        let r = (t as f32 * 2. * PI * frequency as f32 * duration as f32) / n_samples as f32;
        let amplitude = (r.sin() * 10000.) as i16;

        // Data consists  of left channnel followed by right channel sample. As we are generating stereo
        // with both left and right channel being the same, two identical samples are written each time.
        data.push(amplitude );
        data.push(amplitude );
        
    }

    println!("Number of samples {}", data.len());

    let out_header = Header::new(wav::header::WAV_FORMAT_PCM, 2, sampling_rate, 16);


    let mut out_file = File::create("sine_500.wav").expect("Unable to create a wav file");
    wav::write(out_header, &wav::BitDepth::Sixteen(data), &mut out_file).expect("Unable to write to wav file");


    println!("Finished writing");
}
