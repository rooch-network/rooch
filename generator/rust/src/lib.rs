#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(any(feature = "std", test)), no_main)]

mod constants;
mod types;
mod utils;
mod stack_alloc;

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
use debug::*;

#[cfg_attr(not(any(feature = "std", feature = "debug")), panic_handler)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::slice;
use constants::{MAX_CONTENT_SIZE, MAX_STRING_LEN, MAX_DEPLOY_ARGS};
use heapless::{LinearMap, String, Vec};
use types::{Content, DeployArgs, InputData, OutputData, Value};
use utils::Buffer;

fn hash_str_uint32(input: &str) -> u32 {
    let mut hash = 0x811c9dc5;
    let prime = 0x1000193;

    for c in input.chars() {
        let value = c as u8;
        hash ^= value as u32;
        hash = hash.wrapping_mul(prime);
    }

    hash
}

fn get_data_length(buffer: *const u8) -> u32 {
    unsafe {
        let length_bytes = slice::from_raw_parts(buffer, 4);
        u32::from_be_bytes([length_bytes[0], length_bytes[1], length_bytes[2], length_bytes[3]])
    }
}

fn int_to_bytes(n: u32) -> [u8; 4] {
    let bytes = [
        ((n >> 24) & 0xFF) as u8,
        ((n >> 16) & 0xFF) as u8,
        ((n >> 8) & 0xFF) as u8,
        (n & 0xFF) as u8,
    ];
    bytes
}

pub fn inscribe_generate_rust(input: &[u8]) -> Buffer<MAX_CONTENT_SIZE> {
    #[cfg(feature = "debug")]
    printf!("inscribe_generate_start, input size: {}", input.len());

    let input_data = minicbor::decode::<InputData>(input).unwrap();

    #[cfg(feature = "debug")]
    printf!("input_data seed: {}", input_data.seed.as_str());

    #[cfg(feature = "debug")]
    printf!("input_data user_input: {}", input_data.user_input.as_str());

    let mut seed = String::<MAX_CONTENT_SIZE>::new();
    seed.push_str(input_data.seed.as_str()).unwrap();
    seed.push_str(input_data.user_input.as_str()).unwrap();
    let hash_value = hash_str_uint32(seed.as_str());

    let deploy_args: DeployArgs =
        minicbor::decode::<DeployArgs>(input_data.deploy_args.as_slice()).unwrap();

    let mut json_output = deploy_args.args
        .into_iter()
        .filter(|arg| arg.arg.type_name == "range")
        .map(|arg| {
            let range_min = arg.arg.data.min;
            let range_max = arg.arg.data.max;
            let random_value = range_min + (hash_value as u64 % (range_max - range_min + 1));
            (arg.name, Value::UInt(random_value))
        })
        .collect::<LinearMap<_, _, MAX_DEPLOY_ARGS>>();

    let _ = json_output.insert(
        String::try_from("id").unwrap(),
        Value::String(input_data.user_input.clone()),
    );

    let mut body: Vec<u8, MAX_CONTENT_SIZE> = Vec::new();
    body.extend_from_slice("hello world!".as_bytes()).unwrap();

    let output_data = OutputData {
        amount: 1,
        attributes: Some(json_output),
        content: Some(Content {
            content_type: String::<MAX_STRING_LEN>::try_from("text/plain").unwrap(),
            body: body,
        }),
    };

    let mut buf = Buffer::<MAX_CONTENT_SIZE>::new();
    minicbor::encode(&output_data, &mut buf).unwrap();
    buf
}

#[no_mangle]
pub extern "C" fn inscribe_generate(buffer: *const u8) -> *const u8 {
    let buffer_length = get_data_length(buffer);

    #[cfg(feature = "debug")]
    printf!("input_buffer length: {}", buffer_length);

    let input_buffer_slice = unsafe { slice::from_raw_parts(buffer.add(4), buffer_length as usize) };
    let output_buffer = inscribe_generate_rust(input_buffer_slice);

    #[cfg(feature = "debug")]
    printf!("output_buffer length: {}", output_buffer.len());

    let length_bytes = int_to_bytes(output_buffer.len() as u32);

    let mut result_buffer = Buffer::<{ MAX_CONTENT_SIZE + 4 }>::new();
    result_buffer.extend_from_slice(&length_bytes).unwrap();
    result_buffer.extend_from_slice(&output_buffer.as_slice()).unwrap();
    result_buffer.as_slice().as_ptr()
}

#[no_mangle]
pub extern "C" fn inscribe_verify(buffer: *const u8, inscribe_output_buffer: *const u8) -> bool {
    let inscribe_output = inscribe_generate(buffer);
    let output_len = get_data_length(inscribe_output);

    let inscribe_output_slice = unsafe { slice::from_raw_parts(inscribe_output.add(4), output_len as usize) };
    let inscribe_output_buffer_slice = unsafe { slice::from_raw_parts(inscribe_output_buffer, output_len as usize) };

    inscribe_output_slice == inscribe_output_buffer_slice
}

#[no_mangle]
pub extern "C" fn indexer_generate(buffer: *const u8) -> *const u8 {
    inscribe_generate(buffer)
}
