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
//use std::time::Duration;

// use this if depending on local crate
use libsteg;

pub const MAX: usize = usize::max_value();

#[derive(Debug)]
pub enum StegError {
    BadDecode(String),
    BadEncode(String),
    BadError(String),
}

fn main() -> Result<(), StegError> {

    //prepare arguments and check if proper amount are provided
    let args: Vec<String> = env::args().collect();
    if args.len()!=3 && args.len()!=5 {
        eprintln!("You need to give 2 or 4 arguments!");
        return Ok(());
    }

    //determine thread count
    let mut thread_count = &args[1];
    println!("THREAD COUNT: {}",thread_count);

    match args.len() {
        3 => {
            //thread count from argument and parsing
            thread_count = &args[1];
            let mut thread_count = thread_count.parse::<usize>().unwrap();

            //path from second argument 
            let path_string = args[2].to_string();
            let path = Path::new(&path_string);
            //println!("Input Path: {:?}", path);
            let current_dir = env::current_dir().expect("Current directory not found!");
            println!("Current Directory {:?}", current_dir);

            //vector for storing threads and return values from channel, also mpsc channels
            let mut handles = vec![];
            let mut returns = vec![];
            let (sender, receiver) = mpsc::channel();

            //number of files
            let mut num_files = 0;
            //increment for each file in directory
            for _entry in fs::read_dir(path).expect("Path not found!") {num_files = num_files + 1;}

            //list of files
            let mut file_list: Vec<PathBuf> = Vec::new();
            let mut f_l = &file_list.clone();

            

            //shadowing the number of files

            let mut num_files = 0;
            //sorting for only ppm files
            for entry in fs::read_dir(path).expect("Path not found!") {
                print!("Found an entry\n");
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                // if path.extension().unwrap() == "ppm" {
                //     file_list.push(path);
                //     num_files+=1;
                // }
                file_list.push(path);
                num_files+=1;
            }
            for value in &file_list {println!("PPM File: {:?}", value);}//printing the ppm values
            println!("Got here");
            //index of 
            let index = Arc::new(Mutex::new(0));
            let data = Arc::new(Mutex::new(file_list));
            if thread_count >= num_files {thread_count = num_files;}

            for i in 0..thread_count {
                //cloning sending channel
                let tx = sender.clone();
                let index_copied = index.clone();
                let str_test = data.clone();

                //spawn a thread
                let handle = thread::spawn(move || {

                    
                    let str_list = str_test.lock().unwrap();
                    let mut index_unlocked = index_copied.lock().unwrap();
                    let index_value:usize = *index_unlocked;

                    println!("Index {:?}",index_value);
                    println!("Value at index: {:?}",str_list[*index_unlocked]);

                    let file_path = str_list[*index_unlocked].clone().
                    into_os_string();
                    let file_path= file_path.into_string().unwrap();
                    let send_path = file_path.clone();
                    println!("File path as os String: {:?}",file_path);
                    
                    //create a file here and decode it
                    let ppm = match libsteg::PPM::new(file_path) {
                    Ok(ppm) => ppm,
                    Err(err) => panic!("Error: {:?}", err),
                    };
                    //eprintln!("Height: {}", ppm.header.height);
                    //eprintln!("Width: {}", ppm.header.width);
                    //eprintln!("Pixel Length: {}", ppm.pixels.len());
                    //eprintln!("Available Pixels: {}", ppm.pixels.len() / 8);
                    let v = &ppm.pixels;

                    //decode
                    match decode_message(v) {
                        Ok(message) => println!("{}", message),
                        Err(err) => panic!("UNKNOWN ERROR DECODING!"),
                    }

                    let x = thread::current().id();
                    tx.send((x,send_path))
                        .expect("Error sending message!"); //decode return
                    *index_unlocked+=1;
                });

                handles.push(handle);
            }


            for handle in 0..num_files {returns.push(receiver.recv().unwrap());}

            
            //f_l = file_list.clone();
            
            for ret_val in returns {
                for f_name in f_l{
                    let f_name = f_name.clone().into_os_string();
                    let f_name = f_name.into_string().unwrap();
                    let r_val_str = &ret_val.1;

                    println!("Fname: {}",f_name);
                    if ret_val.1 == *f_name{
                        //println!("Retval: {:?} File Name: {:?}",r_val_str,*f_name);
                        println!("Found a match")
                    }
                    
                }
                println!("Returned Value:(thread,index) {:?}", ret_val)
            }

            // for ret_val in &returns{
            //     println!("Retval at 1: {:?}",ret_val.1);
            // }

            // //let mut final_vec = vec![];
            // let f_list = &file_list;
            // let r_list = &returns;
            // for file_name in f_list{
            //     let f_name = file_name.clone();
            //     for returned in r_list{
            //         println!("File name: {:?} Returned Value: {:?}",f_name,returned);
            //         // if returned.1 == file_name{
            //         //     final_vec.push(returned.0);
            //         // }
            //     }
            // }



            
        }
        5 => {

            //cargo run <numThreads> <message file> <ppm directory> <output directory>

            //print out the current directory
            let current_dir = env::current_dir().expect("Current directory not found!");
            println!("Current Directory {:?}", current_dir);

            //let the message be the input from a file
            let message = match fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(err) => return Err(StegError::BadEncode(err.to_string())),
            };
            println!("Total bytes of message: {}", message.capacity());


            let message = message.as_bytes();
            println!("Message as bytes: {:?}",message);

            //get path from input file
            let path_string = args[3].to_string();
            let path = Path::new(&path_string);
            println!("Path provided {:?}",path);

            let mut total_size:usize = 0;

            let mut file_list: Vec<String> = Vec::new();

            for entry in fs::read_dir(path).expect("Path not found!") {
                //print!("Found an entry\n");
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                let path = path.into_os_string().into_string().unwrap();
                let path_str = path.clone();

                file_list.push(path_str);
                let ppm = match libsteg::PPM::new(path) {
                    Ok(ppm) => ppm,
                    Err(err) => panic!("Error: {:?}", err),
                };
                total_size+=ppm.pixels.len();
                print!(" Pixels: {}\n",ppm.pixels.len());
            }
            println!("Total Size: {} Available Size: {}",total_size,total_size/8);
            let total_size=total_size/8;
            if message.len() > total_size{return Ok(());}
            for e in file_list {println!("File: {}",e);}



            let test = &message[0..];
            println!("test {:?}",test);
            
            let mut value: Vec<u8> = Vec::new();
            for element in test.iter() {
                value.push(*element);
            }
            let finalal = String::from_utf8(value).unwrap();
            println!("{}",finalal);

            //determine size of message/split it up into files
            //give each thread a vector of jobs
            //job has a number(Filename) and a payload (message)
            //encode message to file






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

fn find_first(vector_value: Vec<(usize, String)>) -> usize {
    let mut counter: usize = 0;
    while counter < vector_value.len() {
        if vector_value[counter].0 == MAX {
            println!("Index: {}", counter);
            return counter;
        } else {
            counter += 1;
        }
    }
    return MAX;
}
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
