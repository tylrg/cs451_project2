use std::env;
use std::fs;
use std::io;
use std::fs::File;
use std::io::prelude::*;
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
    let thread_count = &args[1];
    // if args.len()!=3 && args.len()!=5 {
    //     eprintln!("You need to give 2 or 4 arguments!");
    //     return Ok(());
    // }else{
    //     let thread_count = &args[1];
    //     //let thread_count = thread_count.parse::<usize>().unwrap();
    //     println!("THREAD COUNT: {}",thread_count);
    // }

    //determine thread count
    

    match args.len() {
        4 => {
            let message = fs::read_to_string(args[1].clone()).unwrap();
            let ppm_name = args[2].clone();
            let output_name = args[3].clone();
            
            
            let index= 203;
            let swag = pad_zeros_for_file(index);
            println!("Padded : {}",swag);
            
            writeout(message,ppm_name,output_name).unwrap();

            // let mut string_list: Vec<String> = Vec::new();
            // string_list.push(String::from("00001.ppm"));
            // string_list.push(String::from("00002.ppm"));
            // string_list.push(String::from("00003.ppm"));
            // string_list.push(String::from("00004.ppm"));

            // for i in 0..4{
            //     writeout(message.clone(), ppm_name.clone(), string_list[i].clone()).unwrap();
            // }
         
        }
        3 => {
            //thread count from argument and parsing
            //thread_count = &args[1];
            
            let thread_count = thread_count.parse::<usize>().unwrap();
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
            //if thread_count >= num_files {thread_count = num_files;}

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
            let thread_count = thread_count.parse::<usize>().unwrap();
            //cargo run <numThreads> <message file> <ppm directory> <output directory>

            //print out the current directory
            let current_dir = env::current_dir().expect("Current directory not found!");
            println!("Current Directory {:?}", current_dir);

            let mut handles = vec![];

            //let the message be the input from a file //ARGS 2
            let message = match fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(err) => return Err(StegError::BadEncode(err.to_string())),
            };
            println!("Total bytes of message: {}", message.capacity());


            let message = message.as_bytes();
            //println!("Message as bytes: {:?}",message);

            //get path from input file
            let path_string = args[3].to_string(); //ARGS 3 input directory
            let path = Path::new(&path_string);
            println!("Path provided {:?}",path);

            let mut total_size:usize = 0;
            
            

            let mut file_list: Vec<String> = Vec::new();

            for entry in fs::read_dir(path).expect("Path not found!") {
                //println!("Found an entry {:?}",entry);
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                
                if path.extension().unwrap() != "ppm" {continue;}
                let path = path.into_os_string().into_string().unwrap();
                let path_str = path.clone();

                file_list.push(path_str);
                
                let ppm = match libsteg::PPM::new(path) {
                    Ok(ppm) => ppm,
                    Err(err) => panic!("Error: {:?}", err),
                };
                total_size+=ppm.pixels.len();
                //print!(" Pixels: {}\n",ppm.pixels.len());

                //comparison
                
            }
            println!("Total Size: {} Available Size: {}",total_size,total_size/8);
            let total_size=total_size/8;
            


            //if message.len() > total_size{return Ok(());}
            //for e in file_list.clone() {println!("File: {}",e);}

            
            //let largest_file = get_biggest(&file_list);
            let largest_file = file_list[0].clone();
            //println!("Largest File {}",largest_file.clone());
            let file_size = pixel_size(largest_file.clone());
            let output_dir = String::from(&args[4]);

            
            

            //determine size of message/split it up into files
            //give each thread a vector of jobs
            //job has a number(Filename) and a payload (message)
            //encode message to file, while job is not empty,

            // job;
            // for loop in threadcount{
            //     job =;
            //     spawn thread
            // }
            

            //slices are fifo
            
            let mut index = 0;
            //let message_parts_count = total_size/thread_count;
            let mut start_slice = 0;
            let mut end_slice = 0;
            
            
            let mut jobs: Vec<(String,String)> = Vec::new();
            //message //filename


                
            //breaking message into chunks
            while start_slice<message.len() {
                //let file_to_use;

                let min = message.len();
                end_slice += file_size+1;
                if end_slice>min {end_slice=min;}
                end_slice = end_slice/8;
                
                println!("Start of slice: {} and end of slice: {}",start_slice,end_slice);



                let message_fragment = &message[start_slice..end_slice];
                let mut str_builder: Vec<u8> = Vec::new();
                for element in message_fragment.iter() {str_builder.push(*element);}
                let assembled = String::from_utf8(str_builder).unwrap();
                //println!("Adding : {}",assembled);

                let write_name = pad_zeros_for_file(index);
                let write_name=format!("{}/{}",output_dir,write_name);
                let job_value = (assembled,write_name);
                jobs.push(job_value);
                index+=1;
                if index == message.len(){index=0;}


                start_slice+=file_size/8+1;
            }

            println!("Jobs: {}", jobs.len());
            // for job in jobs{
            //     println!("{:?}",job);
            // }      
            
            
            let mut start = 0;
            let last_index = 0;
            for i in 0..thread_count{
                           
                //let pair = (1, true);
                let job = i;
                let j = job.clone();
                let mut job_list: Vec<(String,String)> = Vec::new();

                let mut last_index = (thread_count*i)+thread_count+1;
                //println!("#{}: Last Index {} Length {}",i,last_index,jobs.len());
                if last_index > jobs.len()-1{
                    last_index=jobs.len()-1;
                }
                
                if i != 0{
                    start = thread_count*(i);
                    start+=1;
                }

                
                
                for k in start..last_index{
                    job_list.push(jobs[k].clone());
                }

                //println!("LOOP {} Start: {} End: {}",i+1,start,last_index);


                let out = largest_file.clone();
                let handle = thread::spawn(move || {
                    println!("Spawned thread: #{}",j);
                    //let mut current = job_list.len()-1;
                    //let out = largest_file.clone();
                    while job_list.len() !=0 {
                        println!("Thread #{} :Writing a file {:?} Length of message: {}",i,job_list[job_list.len()-1].1,job_list[job_list.len()-1].0.len()-1);
                       

                        // let ppm = match libsteg::PPM::new(out.clone()) {
                        //     Ok(ppm) => ppm,
                        //     Err(err) => panic!("Error: {:?}", err),
                        // };
                        // let output_bytes = encode_message(&job_list[job_list.len()-1].0.as_str(),&ppm);

                        writeout(job_list[job_list.len()-1].0.clone(),out.clone(),job_list[job_list.len()-1].1.clone()).expect("What went wrong?");    
                        // let mut buffer = File::create(job_list[job_list.len()-1].1.clone()).expect("Could not create file");
                        // let yeah = "yeah";
                        // buffer.write(yeah.as_bytes()).unwrap();
                        job_list.pop();                    
                    }
                });
                handles.push(handle);

                start = last_index+1;
            }
            for thread in handles{thread.join().unwrap();}
        }
        _ => println!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
}


fn encode_message(message: &str, ppm: &libsteg::PPM) -> Result<Vec<u8>, StegError> {
    let mut encoded = vec![0u8; 0];
    println!("GOT INTO ENDCODE! Message length {}",message.len());
    // loop through each character in the message
    // for each character, pull 8 bytes out of the file
    // encode those 8 bytes to hide the character in the message
    // add those 8 bytes to the enocded return value
    // add a trailing \0 after all character encoded
    // output the remainder of the original file

    let mut start_index = 0;
    //println!("Message chars {:?}",message.chars().len());
    for c in message.chars() {
        encoded.extend(&encode_character(
            c,
            &ppm.pixels[start_index..start_index + 8],
        ));
        start_index += 8;
        //println!("{}",start_index);
    }
    
    // we need to add a null character to signify end of
    // message in this encoded image
    // encoded.extend(&encode_character(
    //     '\0',
    //     &ppm.pixels[start_index..start_index + 8],
    // ));

    // start_index += 8;

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

fn writeout(message_file: String,ppm_name: String,output_file_name: String) -> std::io::Result<()> {
    //let mut file = File::create(output_file_name)?;
    
    let ppm = match libsteg::PPM::new(ppm_name) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
    };
    println!("MESSAGE LENGTH IN WRITEOUT {}",message_file.len());
    //println!("ABOUT TO ENCODE {}",output_file_name.clone().as_str());
    let mut buffer = File::create(output_file_name).expect("Could not create file");
   
    match encode_message(&message_file, &ppm) {
                Ok(bytes) => {
                    println!("SUCCESS!");

                    // first write magic number
                     buffer
                         .write(&ppm.header.magic_number)
                         .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");
                    //println!("{}",&ppm.header.magic_number.to_string());
                    //println!("P6");
                     buffer
                         .write(&"\n".as_bytes())
                         .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");
                    //print!("{:?}",&"\n".as_bytes());
                    // then the width
                    buffer
                         .write(ppm.header.width.to_string().as_bytes())
                         .expect("FAILED TO WRITE WIDTH TO STDOUT");
                    //print!("{}",ppm.header.width.to_string());
                    buffer
                        .write(&" ".as_bytes())
                        .expect("FAILED TO WRITE WIDTH TO STDOUT");
                    //print!(" ");
                    // then the height
                    buffer
                        .write(ppm.header.height.to_string().as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");
                    //print!("{}",ppm.header.height.to_string());
                    buffer
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");
                    //print!("\n");
                    // then the color value
                    buffer
                        .write(ppm.header.max_color_value.to_string().as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");
                    //println!("{}",ppm.header.max_color_value.to_string());
                    buffer
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");
                    //print!("{:?}",&"\n".as_bytes());

                    // then the encoded byets
                    buffer
                        .write(&bytes)
                        .expect("FAILED TO WRITE ENCODED BYTES TO STDOUT");
                    
                }
                Err(err) => match err {
                    StegError::BadEncode(s) => panic!(s),
                    _ => panic!("RECEIVED AN UNEXPECTED ERROR WHEN TRYING TO ENCODE MESSAGE"),
                },
            }
    Ok(())
}

fn pad_zeros_for_file(index: usize) -> String{
    let mut ret_val:String = index.to_string();
    while ret_val.len() != 5{
        ret_val = format!("0{}",ret_val);
    }
    ret_val=format!("{}.ppm",ret_val);
    return ret_val;
}
fn pixel_size(ppm_name: String)-> usize{
    let ppm = match libsteg::PPM::new(ppm_name) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
    };
    return ppm.pixels.len();
}