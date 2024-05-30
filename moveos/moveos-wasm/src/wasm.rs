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

pub static mut GLOBAL_MEMORY: Lazy<Arc<Mutex<Option<Memory>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));
/*
TODO:
The WASMInstance must be protected by the locker which owned by the signer
    while we enable the parallel tx execution.
The way we're doing it now is by using this big global lock, but it's too broad.
 */
static mut GLOBAL_INSTANCE_POOL: Lazy<Arc<Mutex<BTreeMap<u64, WASMInstance>>>> =
    Lazy::new(|| Arc::new(Mutex::new(BTreeMap::new())));

pub fn insert_wasm_instance(instance: WASMInstance) -> anyhow::Result<u64> {
    loop {
        unsafe {
            let instance_id: u64 = rand::random();
            let mut instance_pool = match GLOBAL_INSTANCE_POOL.lock() {
                Ok(pool_guard) => pool_guard,
                Err(_) => {
                    return Err(anyhow::Error::msg("get global instance pool failed"));
                }
            };
            if instance_pool.get(&instance_id).is_none() {
                instance_pool.insert(instance_id, instance);
                return Ok(instance_id);
            }
        }
    }
}

pub fn get_instance_pool() -> Arc<Mutex<BTreeMap<u64, WASMInstance>>> {
    unsafe { GLOBAL_INSTANCE_POOL.clone() }
}

pub fn remove_instance(instance_id: u64) -> anyhow::Result<()> {
    unsafe {
        let mut instance_pool = match GLOBAL_INSTANCE_POOL.lock() {
            Ok(pool_guard) => pool_guard,
            Err(_) => {
                return Err(anyhow::Error::msg("get global instance pool failed"));
            }
        };
        instance_pool.remove(&instance_id);
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Env {
    memory: Arc<Mutex<Option<Memory>>>,
}

fn fd_write(env: FunctionEnvMut<Env>, _fd: i32, mut iov: i32, iovcnt: i32, pnum: i32) -> i32 {
    let memory_global = unsafe { GLOBAL_MEMORY.clone() };

    let memory = match memory_global.lock() {
        Ok(v) => v,
        Err(_) => return 0,
    };
    let store_ref = env.as_store_ref();
    let memory_obj = match memory.clone() {
        Some(v) => v,
        None => return 0,
    };
    let memory_view = memory_obj.view(&store_ref);

    let mut ptr_buffer: [u8; 4] = [0; 4];
    let mut len_buffer: [u8; 4] = [0; 4];
    let mut write_buffer = Vec::new();
    let mut number = 0;
    for _ in 0..(iovcnt - 1) {
        let ptr_index = (iov) >> 2;
        let len_index = ((iov) + (4)) >> 2;

        match memory_view.read(ptr_index as u64, ptr_buffer.as_mut_slice()) {
            Ok(_) => {}
            Err(_) => return 0,
        }
        let ptr = i32::from_be_bytes(ptr_buffer);

        match memory_view.read(len_index as u64, len_buffer.as_mut_slice()) {
            Ok(_) => {}
            Err(_) => return 0,
        }
        let len = i32::from_be_bytes(len_buffer);

        for i in 0..(len - 1) {
            let single_char = match memory_view.read_u8((ptr + i) as u64) {
                Ok(v) => v,
                Err(_) => return 0,
            };
            write_buffer.push(single_char);
        }

        iov += 8;
        number += len;
    }

    let ret_index = pnum >> 2;
    let ret_index_bytes: [u8; 4] = number.to_be_bytes();
    match memory_view.write(ret_index as u64, ret_index_bytes.as_slice()) {
        Ok(_) => {}
        Err(_) => return 0,
    }
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

pub fn put_data_on_stack(instance: &mut WASMInstance, data: &[u8]) -> anyhow::Result<i32> {
    let stack_alloc_func = match instance.instance.exports.get_function("stackAlloc") {
        Ok(v) => v,
        Err(_) => return Err(anyhow::Error::msg("get stackAlloc function failed")),
    };

    let data_len = data.len() as i32;
    let result = stack_alloc_func.call(&mut instance.store, vec![I32(data_len + 1)].as_slice())?;
    let return_value = match result.deref().first() {
        None => return Err(anyhow::Error::msg("call stackAlloc function failed")),
        Some(v) => v,
    };
    let offset = match return_value.i32() {
        None => return Err(anyhow::Error::msg("the data of function return is not i32")),
        Some(v) => v,
    };

    let memory = match instance.instance.exports.get_memory("memory") {
        Ok(v) => v,
        Err(_) => return Err(anyhow::Error::msg("memory not found")),
    };
    let memory_view = memory.view(&instance.store);
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
    let module = match Module::new(&store, bytecode) {
        Ok(m) => m,
        Err(e) => {
            return Err(anyhow::Error::msg(e.to_string()));
        }
    };

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
    unsafe { *GLOBAL_MEMORY = Arc::new(Mutex::new(Some(memory.clone()))) };

    Ok(WASMInstance::new(bytecode.clone(), instance, store))
}
