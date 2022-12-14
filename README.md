# wav-gen

![Build](https://github.com/adoble/wav-gen/actions/workflows/build.yml/badge.svg?event=push)

A utility program to generate wave forms as either a `wav` file or a rust array initialisation.

The later can be used for embedded projects where a waveform needs to be generated.


## Example Usage

### Sine wave

To generate a **sine wave** with:
- a frequency of 643 hertz
- a duration of 3 seconds
- an output wav file `sine.wav`


```console
 wav-gen wav sine --frequency 643 --duration 3 sine.wav
```
Note: it is assumed throughout that a `wav-gen` alias has been crated for the executable `wav-gen.exe`

### Sweeping Sine Wave

To generate a sine wave that:
- **sweeps** between the frequencies of 300 hertz and 1000 hertz
- has a duration of 5 seconds
- is in the wav file `sweep.wav`

```console
wav-gen wav sweep --start 300  --finish 1000 --duration 5 sweep.wav
```
### Harmonics

To generate a wave that has a set of harmonics first define the harmonics using a csv file (for example `harmonics.csv`):

```
frequency,amplitude
500.0 , 0.3
700.0 , 0.2
750.0 , 0.1
```
This specifies a wave with harmonics at 500Hz, 700Hz and 750Hz with respective amplitudes 0.3, 0.2 and 0.1.
The amplitudes will be normalised.

Then use:

```console
wav-gen wav harmonics --infile harmonics.csv output_wave_file.wav
```
### Rust Data Arrays

To generate a sine waveform of 500Hz as a rust data array of 44140 words use the following

```console
wav-gen rust sine --frequency 500 --length 44100  ./wave.rs
```

The generated rust source code file looks like:

```rust
pub static DATA: [i16; 1024] = [
   // i16 values
];
```
The i16 values alternate between the left channel first and then the right channel. For stereo, each channel has the same value.

If the structure has been, for instance, generated in the file `wave.rs` then it can be imported with:

```rust
use wave;
```


Alternatively monophonic data arrays can be generated:

```console
wav-gen rust sine --frequency 500 --length 1024 --mono ./wave_mono-rs
```

This works the same for the other wave types such as `sweep` and `harmonics`.

A different name for the rust data structure can be specified:
```console
wav-gen rust sweep --start 500 --finish 1500 --name SWEEP_DATA ./sweep.rs
```

The generated rust source code looks like:

```rust
pub static SWEEP_DATA: [i16; 1024] = [
         0,     0,    71,    71,   143,   143,   214,   214,   285,   285,
       355,   355,   423,   423,   490,   490,   554,   554,   616,   616,
       // ... more i16 values ...
       947,   947,   777,   777,
];
```

For sine waves and harmonics, instead of generating a rust source code file with a large number of samples, only one cycle can be generated
by using the `--cycle` flag, e.g.:


```console
wav-gen rust sine --frequency 2000 --cycle  ./src/SINE_DATA.rs
```

## More options
For more options use:

```console
wav-gen help
```
