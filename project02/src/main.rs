use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str;
use std::thread;

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
    if args.len()!=5 {
        eprintln!("You need to give 2 or 4 arguments!");
        return Ok(());
    }
    

    match args.len() {
        5 => {
            let thread_count = thread_count.parse::<usize>().unwrap();
            //cargo run <numThreads> <message file> <ppm directory> <output directory>

            let mut handles = vec![];//vector for holding thread

            //let the message be the input from a file //ARGS 2
            let mut message = match fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(err) => return Err(StegError::BadEncode(err.to_string())),
            };

            //null terminate message
            let end = vec![0];
            let end = str::from_utf8(&end).unwrap();
            let end:String = String::from(end);
            let end =  end.chars();
            message.push(end.clone().next().unwrap());

            let message = message.as_bytes();//message from input file
            
            //get path from input file
            let path_string = args[3].to_string(); //ARGS 3 input directory
            let path = Path::new(&path_string);//path from string

            let mut total_size:usize = 0;//total size of all files
            
            let mut file_list: Vec<String> = Vec::new();//list of all files

            //finds all ppm files and filters
            for entry in fs::read_dir(path).expect("Path not found!") {
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
            }

            let total_size=total_size/8;//size of space given bytes needed for encoding
            
            if message.len() > total_size{return Ok(());}

            let input_file = file_list[0].clone();//set input file for ppm source

            //eprintln!("Largest File {}",input_file.clone());
            let file_size = pixel_size(input_file.clone());//soze of the file
            let output_dir = String::from(&args[4]);//output directory

            let mut index = 0;//keeps track of next file name
            
            let mut start_slice = 0;//start of slice
            let mut end_slice = 0;//end of slices
            
            let mut jobs: Vec<(String,String)> = Vec::new();//all possible jobs
            //(message,filename)

            //while we still have slices of messages left to allocate
            while start_slice<message.len() {


                let min = message.len();//file length for comparison
                end_slice = end_slice+file_size/8;//end of the slice for reading and writing
                if end_slice>min {end_slice=min;}//set end to old end or message length, minimum

                let message_fragment = &message[start_slice..end_slice];//message fragment to decode
                let mut str_builder: Vec<u8> = Vec::new();//beginning of string
                for element in message_fragment.iter() {str_builder.push(*element);}//assemble string for building
                let assembled = String::from_utf8(str_builder).unwrap();//assemble string
                

                let write_name = pad_zeros_for_file(index);//pad file name to zeros
                let write_name=format!("{}/{}",output_dir,write_name);//format with directory name
                let job_value = (assembled,write_name);
                jobs.push(job_value);//add job to list of jobs
                index+=1;
            
                start_slice=end_slice;
            }

            //for each thread...
            for i in 0..thread_count{
                        
                let mut job_list: Vec<(String,String)> = Vec::new();//initialize job list
                let decimal_length: f64 = jobs.len() as f64;
                let interval = (decimal_length/thread_count as f64).ceil();
                let interval: usize = interval as usize; //determine interval size
                let start =  interval*i; //determine start index for this threads jobs
                let mut last_index = start+interval; //set last index as interval distance from start
                if last_index>=jobs.len() {last_index=jobs.len();} // if last is greater than number of files, set to number of files -1

                let mut counter = start;//counter for which job to add

                //until the job list is of properlength(), add jobs
                while job_list.len()<interval{
                    if counter >= last_index {break;}//if counter is greater than index, dont' add
                    //eprintln!("Thread: {} is getting {}, responsible for {}/{}",i,counter,start,last_index-1);
                    job_list.push(jobs[counter].clone());//push the path to the job list
                    counter+=1;//increment
                }

                //output file to write out to
                let out = input_file.clone();

                //spawn a thread
                let handle = thread::spawn(move || {
                    while job_list.len() !=0 {                        
                        writeout(job_list[job_list.len()-1].0.clone(),out.clone(),job_list[job_list.len()-1].1.clone()).expect("Could not write out");    

                        job_list.pop();//pop job off queue         
                    }
                });
                handles.push(handle);//add thread to handle

                
            }
            for thread in handles{thread.join().unwrap();}//wait for each thread
        }
        _ => eprintln!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
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
    
    //i needed to get rid of this, there is some extra junk printed as a result
    // we need to add a null character to signify end of
    // message in this encoded image
    // encoded.extend(&encode_character(
    //     '\0',
    //     &ppm.pixels[start_index..start_index + 8],
    // ));

    //start_index += 8;

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

fn writeout(message_file: String,ppm_name: String,output_file_name: String) -> std::io::Result<()> {
    //let mut file = File::create(output_file_name)?;
    
    let ppm = match libsteg::PPM::new(ppm_name) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
    };

    let mut buffer = File::create(output_file_name).expect("Could not create file");
   
    match encode_message(&message_file, &ppm) {
                Ok(bytes) => {
                    // first write magic number
                     buffer
                         .write(&ppm.header.magic_number)
                         .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");

                     buffer
                         .write(&"\n".as_bytes())
                         .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");

                    buffer
                         .write(ppm.header.width.to_string().as_bytes())
                         .expect("FAILED TO WRITE WIDTH TO STDOUT");

                    buffer
                        .write(&" ".as_bytes())
                        .expect("FAILED TO WRITE WIDTH TO STDOUT");

                    buffer
                        .write(ppm.header.height.to_string().as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");

                    buffer
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");
                    
                    buffer
                        .write(ppm.header.max_color_value.to_string().as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");

                    buffer
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");

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

//pads a file to given length with zeros
fn pad_zeros_for_file(index: usize) -> String{
    let mut ret_val:String = index.to_string();
    while ret_val.len() != 5{
        ret_val = format!("0{}",ret_val);
    }
    ret_val=format!("{}.ppm",ret_val);
    return ret_val;
}

//gets the size of pixels for a given string
fn pixel_size(ppm_name: String)-> usize{
    let ppm = match libsteg::PPM::new(ppm_name) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
    };
    return ppm.pixels.len();
}