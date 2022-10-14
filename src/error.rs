/// Error handling
use std::fmt;
use std::path::PathBuf;

pub enum WavGenError {
    ReadError(PathBuf),
    WriteError(PathBuf),
    CreateError(PathBuf),
    HarmonicParseError(usize),
    NoHarmonics,
}

//Required for the ? operator
impl std::error::Error for WavGenError {}

// Formatted display for the errors
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

// Using the display implementation for the debug implementation means that
// user friendly messages are shown when the main funtion exists with
// an error
impl std::fmt::Debug for WavGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <WavGenError as fmt::Display>::fmt(self, f)
    }
}
