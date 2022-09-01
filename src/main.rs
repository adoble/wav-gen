use std::fs::File;
use std::io::prelude::*;


fn main() {

    let data = "Even more data!";

    let mut out_file = File::create("output.txt").expect("Unable to crate a file");
    out_file.write_all(data.as_bytes()).expect("Unable to write data to file");


    println!("Finished writing");
}
