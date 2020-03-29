use std::env;
use std::fs;
//use std::io;
//use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
//use std::rc::Rc;
use std::str;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// use this if depending on local crate
use libsteg;
#[derive(Debug)]
pub enum StegError {
    BadDecode(String),
    BadEncode(String),
    BadError(String),
}

fn main() -> Result<(), StegError> {
    let args: Vec<String> = env::args().collect();
    let mut thread_count;

    if args.len() > 2 {
        thread_count = &args[1];
        println!("THREADS TO BE USED: {}", thread_count);
    } else {
        eprintln!("You need to give 2 or 4 arguments!");
        //return Ok(());
    }

    match args.len() {
        2 => {
             thread_count = &args[1];
            // let (sender, receiver ) = mpsc::channel();
            // let mut handles = vec![];
            // let mut values = vec![];
            // let data = Arc::new(Mutex::new(vec![1, 2, 3,4,5,6,7,8,9,10,1, 2, 3,4,5,6,7,8,9,10,1, 2, 3,4,5,6,7,8,9,10]));
            // //let mut workload = Arc::new(vec![1, 2, 3]);

            // let mut index = Arc::new(Mutex::new(0 as usize));
            // for i in 0..thread_count.parse::<i32>().unwrap(){
            //     let tx = sender.clone();
            //     let data2 = data.clone();
            //     let id = index.clone();
            //     let handle = thread::spawn(move||{
            //         let x = thread::current().id();
            //         let mut data = data2.lock().unwrap();
            //         let index2:usize = index.lock().unwrap();
            //         //data[i]+=1;
            //         //let mut num = val.lock().unwrap();
            //         //let y =2000*4;
            //         tx.send((x,data[index2]));
            //         *index2+=1;
            //         });
            //     handles.push(handle);
            // }

            // for handle in handles{values.push(receiver.recv().unwrap());}
            // println!("Values: {:?}",values)}
            let mut handles = vec![];
            let (sender, receiver ) = mpsc::channel();
            let mut values = vec![];
            let mut returns = vec![];

            //directory size up
            for i in 1..32{
                let none_val:usize = 0usize;
                values.push((none_val,String::from("File at: ")+&i.to_string()));
            }
            println!("Values: {:?}",values);

            let data = Arc::new(Mutex::new(values));
            //if thread_count >= values.len(){thread_count=values.len()}
            for i in 1..thread_count.parse::<usize>().unwrap()+1 {
                let data = data.clone();
                let tx = sender.clone();
                let handle = thread::spawn(move || {
                    let mut data = data.lock().unwrap();
                    if i < data.len(){
                        if data[i].0 == 0{
                            //println!("Not consumed: {:?}",i);
                            //println!("Value at {}: {:?}",i,data[i]);
                            //thread::sleep(Duration::from_millis(50));
                            data[i].0 = i;
                            //println!("Updated Value at {}: {:?}\n",i,data[i]);
                            tx.send((data[i].0,data[i].1.clone()));
                            thread::sleep(Duration::from_millis(50));
                        }
                    }

                    
                });
                handles.push(handle);
            }
            println!("Length of Handles(Number of threads): {}",handles.len());

            for handle in handles{returns.push(receiver.recv().unwrap());}
            for ret_val in returns{println!("Returned Value: {:?}",ret_val)};
        }
        3 => {
            //let (tx, rx) = mpsc::channel();
            let mut num_files = 0;

            //let mut file_list: Vec<str> = Vec::new();
            let path_string = args[2].to_string();
            let path = Path::new(&path_string);
            print!("Input Path: {:?}", path);
            let current_dir = env::current_dir().expect("Fuck");
            println!(
                "Entries modified in the last 24 hours in {:?}:",
                current_dir
            );

            //is dir
            for _entry in fs::read_dir(path).expect("Path not found!") {
                num_files = num_files + 1;
            }
            println!("Number of files: {}", num_files);
            let mut file_list: Vec<PathBuf> = Vec::new();
            let mut str_parts = vec![" "; 0];
            for entry in fs::read_dir(path).expect("Path not found!") {
                let entry = entry.expect("Why do I even need this?");

                //let path_value = entry.path();
                let path = entry.path();
                if path.extension().unwrap() == "ppm" {
                    file_list.push(path);
                    str_parts.push(" ");
                }
            }
            // let mut str_parts: Vec<str> = Vec::new();
            for value in &file_list {
                println!("Value: {:?}", value);
                //str_parts.push(" ");
            }
            println!("Length of str_parts: {}", str_parts.len());

            // let counter = Arc::new(Mutex::new(0));
            // let mut handles = vec![];
            // let mut mutex_file_list = Arc::new(Mutex::new(file_list));
            // for _ in 0..thread_count.parse::<i32>().unwrap() {
            //     let counter = Arc::clone(&counter);
            //     let handle = thread::spawn(move || {
            //         let mut num = counter.lock().unwrap();
            //         println!("List at {:?}",file_list[0]);
            //         *num += 1;
            //         println!("Counter: {}",num);
            //     });
            //     handles.push(handle);
            // }
            // for handle in handles {
            //     handle.join().unwrap();
            // }
            // println!("Result: {}", *counter.lock().unwrap())

            // let ppm = match libsteg::PPM::new(args[2].to_string()) {
            //     Ok(ppm) => ppm,
            //     Err(err) => panic!("Error: {:?}", err),
            // };
            // eprintln!("Height: {}", ppm.header.height);
            // eprintln!("Width: {}", ppm.header.width);
            // eprintln!("Pixel Length: {}", ppm.pixels.len());
            // eprintln!("Available Pixels: {}", ppm.pixels.len() / 8);
            // let v = &ppm.pixels;
            // match decode_message(v) {
            //     Ok(message) => println!("{}", message),
            //     Err(err) => panic!("UNKNOWN ERROR DECODING!"),
            // }
        }
        5 => {
            let message = match fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(err) => return Err(StegError::BadEncode(err.to_string())),
            };

            eprintln!("Total bytes of message: {}", message.capacity());

            // let ppm = match libsteg::PPM::new(args[].to_string()) {
            //     Ok(ppm) => ppm,
            //     Err(err) => panic!("Error: {:?}", err),
            // };

            // match encode_message(&message, &ppm) {
            //     Ok(bytes) => {
            //         // we got some bytes
            //         // need to write ppm header first
            //         // TODO move this to library

            //         // first write magic number
            //         io::stdout()
            //             .write(&ppm.header.magic_number)
            //             .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");
            //         io::stdout()
            //             .write(&"\n".as_bytes())
            //             .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");

            //         // then the width
            //         io::stdout()
            //             .write(ppm.header.width.to_string().as_bytes())
            //             .expect("FAILED TO WRITE WIDTH TO STDOUT");
            //         io::stdout()
            //             .write(&" ".as_bytes())
            //             .expect("FAILED TO WRITE WIDTH TO STDOUT");

            //         // then the height
            //         io::stdout()
            //             .write(ppm.header.height.to_string().as_bytes())
            //             .expect("FAILED TO WRITE HEIGHT TO STDOUT");
            //         io::stdout()
            //             .write(&"\n".as_bytes())
            //             .expect("FAILED TO WRITE HEIGHT TO STDOUT");

            //         // then the color value
            //         io::stdout()
            //             .write(ppm.header.max_color_value.to_string().as_bytes())
            //             .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");
            //         io::stdout()
            //             .write(&"\n".as_bytes())
            //             .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");

            //         // then the encoded byets
            //         // io::stdout()
            //         //     .write(&bytes)
            //         //     .expect("FAILED TO WRITE ENCODED BYTES TO STDOUT");
            //     }
            //     Err(err) => match err {
            //         StegError::BadEncode(s) => panic!(s),
            //         _ => panic!("RECEIVED AN UNEXPECTED ERROR WHEN TRYING TO ENCODE MESSAGE"),
            //     },
            // }
        }
        _ => println!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
}

pub struct DecodeFragment {}

fn encode_message(message: &str, ppm: &libsteg::PPM) -> Result<Vec<u8>, StegError> {
    let mut encoded = vec![0u8; 0];

    // loop through each character in the message
    // for each character, pull 8 bytes out of the file
    // encode those 8 bytes to hide the character in the message
    // add those 8 bytes to the enocded return value
    // add a trailing \0 after all character encoded
    // output the remainder of the original file

    let mut start_index = 0;
    for c in message.chars() {
        encoded.extend(&encode_character(
            c,
            &ppm.pixels[start_index..start_index + 8],
        ));
        start_index += 8;
    }

    // we need to add a null character to signify end of
    // message in this encoded image
    encoded.extend(&encode_character(
        '\0',
        &ppm.pixels[start_index..start_index + 8],
    ));

    start_index += 8;

    // spit out remainder of ppm pixel data.
    encoded.extend(&ppm.pixels[start_index..]);

    Ok(encoded)
}
fn encode_character(c: char, bytes: &[u8]) -> [u8; 8] {
    let c = c as u8;

    let mut ret = [0u8; 8];

    for i in 0..bytes.len() {
        if bit_set_at(c, i) {
            ret[i] = bytes[i] | 00000_0001;
        } else {
            ret[i] = bytes[i] & 0b1111_1110;
        }
    }

    ret
}
fn bit_set_at(c: u8, position: usize) -> bool {
    bit_at(c, position) == 1
}
fn bit_at(c: u8, position: usize) -> u8 {
    (c >> (7 - position)) & 0b0000_0001
}
fn decode_message(pixels: &Vec<u8>) -> Result<String, StegError> {
    let mut message = String::from("");

    for bytes in pixels.chunks(8) {
        // eprintln!("chunk!");
        if bytes.len() < 8 {
            panic!("There were less than 8 bytes in chunk");
        }

        let character = decode_character(bytes);

        if character > 127 {
            return Err(StegError::BadDecode(
                "Found non-ascii value in decoded character!".to_string(),
            ));
        }

        message.push(char::from(character));

        if char::from(character) == '\0' {
            // eprintln!("Found terminating null!");
            break;
        }
    }

    Ok(message)
}
fn decode_character(bytes: &[u8]) -> u8 {
    if bytes.len() != 8 {
        panic!("Tried to decode from less than 8 bytes!");
    }

    let mut character: u8 = 0b0000_0000;

    for (i, &byte) in bytes.iter().enumerate() {
        if lsb(byte) {
            match i {
                0 => character ^= 0b1000_0000,
                1 => character ^= 0b0100_0000,
                2 => character ^= 0b0010_0000,
                3 => character ^= 0b0001_0000,
                4 => character ^= 0b0000_1000,
                5 => character ^= 0b0000_0100,
                6 => character ^= 0b0000_0010,
                7 => character ^= 0b0000_0001,
                _ => panic!("uh oh!"),
            }
        }
    }

    character
}
fn lsb(byte: u8) -> bool {
    (0b0000_0001 & byte) == 1
}
