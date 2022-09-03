# wav-gen

## wav-gen

### Introduction

A utility program to generate wave forms as a `wav` file.


### Example Usage

To generate a **sine wave** with:
- a frequency of 643 hertz
- a duration of 3 seconds
- an output wav file `sine.wav`


```console
 wav-gen sine --frequency 643 --duration 3 sine.wav
```
To generate a sine wave that:
- **sweeps** between the frequencies of 300 hertz and 1000 hertz
- has a duration of 5 seconds
- is in the wav file `sweep.wav`

```console
wav-gen sweep --start 300  --finish 1000 --duration 5 sweep.wav
```

For more options use:

```console
wav-gen help
```
