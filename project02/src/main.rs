use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::str;

// use this if depending on local crate
use libsteg;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let ppm = match libsteg::PPM::new(args[1].to_string()) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
            };

            let v = &ppm.pixels;

            match decode_message(v) {
                Ok(message) => println!("{}", message),
                Err(err) => panic!("UNKNOWN ERROR DECODING!"),
                },
            }
        }
        3 => {
            let message = match fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(err) => return Err(StegError::BadEncode(err.to_string())),
            };

            let ppm = match libsteg::PPM::new(args[1].to_string()) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
            };

            match encode_message(&message, &ppm) {
                Ok(bytes) => {
                    // we got some bytes
                    // need to write ppm header first
                    // TODO move this to library

                    // first write magic number
                    io::stdout()
                        .write(&ppm.header.magic_number)
                        .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");
                    io::stdout()
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");

                    // then the width
                    io::stdout()
                        .write(ppm.header.width.to_string().as_bytes())
                        .expect("FAILED TO WRITE WIDTH TO STDOUT");
                    io::stdout()
                        .write(&" ".as_bytes())
                        .expect("FAILED TO WRITE WIDTH TO STDOUT");

                    // then the height
                    io::stdout()
                        .write(ppm.header.height.to_string().as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");
                    io::stdout()
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");

                    // then the color value
                    io::stdout()
                        .write(ppm.header.max_color_value.to_string().as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");
                    io::stdout()
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");

                    // then the encoded byets
                    io::stdout()
                        .write(&bytes)
                        .expect("FAILED TO WRITE ENCODED BYTES TO STDOUT");
                }
                Err(err) => match err {
                    StegError::BadEncode(s) => panic!(s),
                    _ => panic!("RECEIVED AN UNEXPECTED ERROR WHEN TRYING TO ENCODE MESSAGE"),
                },
            }
        }

        _ => println!("You need to give one or two arguments!"),
    }

    Ok(())