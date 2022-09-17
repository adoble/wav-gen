# wav-gen

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
### Rust data arrays

To generate a sine waveform of 500Hz as a rust data array of 44140 words use the following

```console
wav-gen rust sine --frequency 500 --length 44100  ./wave.rs
```

The generated rust source code file looks like:

```rust
static DATA: [i16; 1024] = [
   // i16 values
];
```
The i16 values alternate between the left channel first and then the right channel. For stereo, each channel has the same value.

A monophonic data array can be generated:

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
static SWEEP_DATA: [i16; 1024] = [
         0,     0,    71,    71,   143,   143,   214,   214,   285,   285,
       355,   355,   423,   423,   490,   490,   554,   554,   616,   616,
       // ... more i16 values ...
       947,   947,   777,   777,
];
```

Notes:

* Monophonic data represents twice the duration as for stereophonic data as the samples are not repeated for each channel.
* For the rust output type any duration specified is an error.

## More options
For more options use:

```console
wav-gen help
```
