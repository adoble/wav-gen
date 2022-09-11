# wav-gen


A utility program to generate wave forms as a `wav` file.


## Example Usage

### Sine wave

To generate a **sine wave** with:
- a frequency of 643 hertz
- a duration of 3 seconds
- an output wav file `sine.wav`


```console
 wav-gen sine --frequency 643 --duration 3 sine.wav
```

### Sweeping Sine Wave

To generate a sine wave that:
- **sweeps** between the frequencies of 300 hertz and 1000 hertz
- has a duration of 5 seconds
- is in the wav file `sweep.wav`

```console
wav-gen sweep --start 300  --finish 1000 --duration 5 sweep.wav
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
wav-gen -m harmonics.csv output_wave_file.wav
```

## More options
For more options use:

```console
wav-gen help
```
