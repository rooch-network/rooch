// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use rand;
use wasmer::Value::I32;
use wasmer::*;

//#[derive(Clone)]
pub struct WASMInstance {
    pub bytecode: Vec<u8>,
    pub instance: Instance,
    pub store: Store,
}

impl WASMInstance {
    pub fn new(bytecode: Vec<u8>, instance: Instance, store: Store) -> Self {
        Self {
            bytecode,
            instance,
            store,
        }
    }
}

pub static mut GLOBAL_MEMORY: Lazy<Option<Arc<Mutex<Memory>>>> = Lazy::new(|| None);
static mut GLOBAL_INSTANCE_POOL: Lazy<Arc<Mutex<BTreeMap<u64, WASMInstance>>>> =
    Lazy::new(|| Arc::new(Mutex::new(BTreeMap::new())));

pub fn insert_wasm_instance(instance: WASMInstance) -> u64 {
    loop {
        unsafe {
            let instance_id: u64 = rand::random();
            if GLOBAL_INSTANCE_POOL
                .lock()
                .unwrap()
                .get(&instance_id)
                .is_none()
            {
                GLOBAL_INSTANCE_POOL
                    .lock()
                    .unwrap()
                    .insert(instance_id, instance);
                return instance_id;
            }
        }
    }
}

pub fn get_instance_pool() -> Arc<Mutex<BTreeMap<u64, WASMInstance>>> {
    unsafe { GLOBAL_INSTANCE_POOL.clone() }
}

pub fn remove_instance(instance_id: u64) {
    unsafe {
        GLOBAL_INSTANCE_POOL.lock().unwrap().remove(&instance_id);
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Env {
    memory: Option<Arc<Mutex<Memory>>>,
}

fn fd_write(env: FunctionEnvMut<Env>, _fd: i32, mut iov: i32, iovcnt: i32, pnum: i32) -> i32 {
    let memory_obj = unsafe { GLOBAL_MEMORY.clone().unwrap() };

    // let binding = env.data().memory.clone().unwrap();
    // let memory = binding.lock().unwrap();
    let memory = memory_obj.lock().expect("getting memory mutex failed");
    let store_ref = env.as_store_ref();
    let memory_view = memory.view(&store_ref);

    let mut temp_buffer: [u8; 4] = [0; 4];
    let mut number = 0;
    for _ in 0..(iovcnt - 1) {
        let ptr_index = (iov) >> 2;
        let len_index = ((iov) + (4)) >> 2;

        memory_view
            .read(ptr_index as u64, temp_buffer.as_mut_slice())
            .expect("read data from memory view failed");
        let _ptr = i32::from_be_bytes(temp_buffer);

        memory_view
            .read(len_index as u64, temp_buffer.as_mut_slice())
            .expect("read data from memory view failed");
        let len = i32::from_be_bytes(temp_buffer);

        iov += 8;
        number += len;
    }

    let ret_index = (pnum) >> 2;
    let ret_index_bytes: [u8; 4] = number.to_be_bytes();
    memory_view
        .write(ret_index as u64, ret_index_bytes.as_slice())
        .expect("write data to memory failed");
    0
}

fn convert_i32_pair_to_i53_checked(lo: i32, hi: i32) -> i32 {
    let p0 = if lo > 0 { 1 } else { 0 };
    let p1 = (hi + 0x200000) >> p0 < (0x400001 - p0);
    if p1 {
        let (e0, _) = (hi as u32).overflowing_add_signed(429496729);
        let (e1, _) = (lo >> 1).overflowing_add_unsigned(e0);
        e1
    } else {
        0
    }
}

fn fd_seek(
    _env: FunctionEnvMut<Env>,
    _fd: i32,
    offset_low: i64,
    offset_high: i32,
    _whence: i32,
) -> i32 {
    let _offset = convert_i32_pair_to_i53_checked(offset_low as i32, offset_high);
    70
}

fn fd_close(_env: FunctionEnvMut<Env>, _fd: i32) -> i32 {
    0
}

fn proc_exit(_env: FunctionEnvMut<Env>, code: i32) {
    eprintln!("program exit with {:}", code)
}

pub fn put_data_on_stack(
    stack_alloc_func: &Function,
    store: &mut Store,
    data: &[u8],
) -> anyhow::Result<i32> {
    let data_len = data.len() as i32;
    let result = stack_alloc_func.call(store, vec![I32(data_len + 1)].as_slice())?;
    let return_value = match result.deref().get(0) {
        None => return Err(anyhow::Error::msg("call StaclAlloc function failed")),
        Some(v) => v,
    };
    let offset = match return_value.i32() {
        None => return Err(anyhow::Error::msg("the data of function return is not i32")),
        Some(v) => v,
    };

    let memory = unsafe { GLOBAL_MEMORY.clone().expect("global memory is none") };

    let bindings = match memory.lock() {
        Ok(v) => v,
        Err(_) => return Err(anyhow::Error::msg("memory lock failed")),
    };
    let memory_view = bindings.view(store);
    memory_view.write(offset as u64, data)?;

    Ok(offset)
}

pub fn get_data_from_heap(
    memory: Arc<Mutex<Memory>>,
    store: &Store,
    ptr_offset: i32,
) -> anyhow::Result<Vec<u8>> {
    let bindings = match memory.lock() {
        Ok(v) => v,
        Err(_) => return Err(anyhow::Error::msg("memory lock failed")),
    };
    let memory_view = bindings.view(store);
    let mut length_bytes: [u8; 4] = [0; 4];
    match memory_view.read(ptr_offset as u64, length_bytes.as_mut_slice()) {
        Ok(_) => {}
        Err(_) => return Err(anyhow::Error::msg("read memory failed")),
    }
    let length = u32::from_be_bytes(length_bytes);
    let mut data = vec![0; length as usize];
    match memory_view.read((ptr_offset + 4) as u64, &mut data) {
        Ok(_) => {}
        Err(_) => return Err(anyhow::Error::msg("read memory failed")),
    }
    Ok(data)

    // let ptr = memory_view.data_ptr().offset(ptr_offset as isize) as *mut c_char;
    // let c_str = CStr::from_ptr(ptr);
    // c_str.to_bytes().to_vec()
    // let rust_str = c_str.to_str().expect("Bad encoding");
    // let owned_str = rust_str.to_owned();
    // owned_str
}

pub fn create_wasm_instance(bytecode: &Vec<u8>) -> anyhow::Result<WASMInstance> {
    let mut store = Store::default();
    let module = Module::new(&store, bytecode).unwrap();

    let global_memory = unsafe { GLOBAL_MEMORY.clone() };
    let env = FunctionEnv::new(
        &mut store,
        Env {
            memory: global_memory,
        },
    );

    let import_object = imports! {
        "wasi_snapshot_preview1" => {
            "fd_write" => Function::new_typed_with_env(&mut store, &env, fd_write),
            "fd_seek" => Function::new_typed_with_env(&mut store, &env, fd_seek),
            "fd_close" => Function::new_typed_with_env(&mut store, &env, fd_close),
            "proc_exit" => Function::new_typed_with_env(&mut store, &env, proc_exit),
        }
    };

    let instance = match Instance::new(&mut store, &module, &import_object) {
        Ok(v) => v,
        Err(_) => return Err(anyhow::Error::msg("create wasm instance failed")),
    };
    let memory = match instance.exports.get_memory("memory") {
        Ok(v) => v,
        Err(_) => return Err(anyhow::Error::msg("get memory failed")),
    };
    unsafe { *GLOBAL_MEMORY = Some(Arc::new(Mutex::new(memory.clone()))) };

    Ok(WASMInstance::new(bytecode.clone(), instance, store))
}
