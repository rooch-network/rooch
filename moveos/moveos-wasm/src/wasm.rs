// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use rand;
use tracing::{debug, error, warn};
use wasmer::Value::I32;
use wasmer::*;

use crate::gas_meter::GasMeter;
use crate::middlewares::gas_metering::GasMiddleware;

const GAS_LIMIT: u64 = 10000;

//#[derive(Clone)]
pub struct WASMInstance {
    pub bytecode: Vec<u8>,
    pub instance: Instance,
    pub store: Store,
    pub gas_meter: Arc<Mutex<GasMeter>>,
}

impl WASMInstance {
    pub fn new(
        bytecode: Vec<u8>,
        instance: Instance,
        store: Store,
        gas_meter: Arc<Mutex<GasMeter>>,
    ) -> Self {
        Self {
            bytecode,
            instance,
            store,
            gas_meter,
        }
    }
}

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
    memory: Option<Arc<Mutex<Memory>>>,
    gas_meter: Arc<Mutex<GasMeter>>,
}

fn js_log(env: FunctionEnvMut<Env>, ptr: i32, len: i32) {
    if let Some(memory_obj) = env.data().memory.clone() {
        let memory = memory_obj.lock().expect("getting memory mutex failed");
        let store_ref = env.as_store_ref();
        let memory_view = memory.view(&store_ref);

        let mut buffer = vec![0u8; len as usize];
        memory_view
            .read(ptr as u64, &mut buffer)
            .expect("read buffer from memory failed");

        let message = String::from_utf8_lossy(&buffer);
        debug!("js_log_output: {}", message);
    }
}

fn fd_write(env: FunctionEnvMut<Env>, _fd: i32, mut iov: i32, iovcnt: i32, pnum: i32) -> i32 {
    let mut written_bytes = 0;

    if let Some(memory_obj) = env.data().memory.clone() {
        debug!(
            "fd_write: fd:{}, iov:{}, iovcnt:{}, pnum:{}",
            _fd, iov, iovcnt, pnum
        );

        let memory = memory_obj.lock().expect("getting memory mutex failed");
        let store_ref = env.as_store_ref();
        let memory_view = memory.view(&store_ref);

        let mut temp_buffer: [u8; 4] = [0; 4];

        for _ in 0..iovcnt {
            let ptr_index = iov;
            let len_index = iov + 4;

            memory_view
                .read(ptr_index as u64, temp_buffer.as_mut_slice())
                .expect("read data from memory view failed");
            let _ptr = u32::from_le_bytes(temp_buffer);

            memory_view
                .read(len_index as u64, temp_buffer.as_mut_slice())
                .expect("read data from memory view failed");
            let len = u32::from_le_bytes(temp_buffer);

            debug!("fd_write: _ptr:{}, len:{}", _ptr, len);

            let mut buffer = vec![0u8; len as usize];
            memory_view
                .read(_ptr as u64, &mut buffer)
                .expect("read buffer from memory failed");

            match _fd {
                // stdout
                1 => {
                    use std::io::{self, Write};
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(&buffer).expect("write to stdout failed");
                    debug!("fd_write_stdout: {}", String::from_utf8_lossy(&buffer));
                }
                // stderr
                2 => {
                    use std::io::{self, Write};
                    let stderr = io::stderr();
                    let mut handle = stderr.lock();
                    handle.write_all(&buffer).expect("write to stderr failed");
                    warn!("fd_write_stderr: {}", String::from_utf8_lossy(&buffer));
                }
                // Handle other file descriptors...
                _ => unimplemented!(),
            }

            iov += 8;
            written_bytes += len as i32;
        }

        let ret_index = pnum;
        let ret_index_bytes: [u8; 4] = written_bytes.to_le_bytes();
        memory_view
            .write(ret_index as u64, ret_index_bytes.as_slice())
            .expect("write data to memory failed");
    }

    written_bytes
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
    error!("program exit with {:}", code)
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
    memory: &mut Arc<Mutex<Memory>>,
    store: &Store,
    ptr_offset: i32,
) -> Vec<u8> {
    let bindings = memory.lock().expect("getting memory mutex failed");
    let memory_view = bindings.view(store);
    let mut length_bytes: [u8; 4] = [0; 4];
    memory_view
        .read(ptr_offset as u64, length_bytes.as_mut_slice())
        .expect("read length_bytes failed");
    let length = u32::from_be_bytes(length_bytes);
    let mut data = vec![0; length as usize];
    memory_view
        .read((ptr_offset + 4) as u64, &mut data)
        .expect("read uninit failed");
    data
}

fn charge(env: FunctionEnvMut<Env>, amount: i64) -> Result<(), wasmer::RuntimeError> {
    let mut gas_meter = env.data().gas_meter.lock().unwrap();
    gas_meter.charge(amount as u64)
}

pub fn create_wasm_instance(code: &[u8]) -> anyhow::Result<WASMInstance> {
    // Create the GasMeter
    let gas_meter = Arc::new(Mutex::new(GasMeter::new(GAS_LIMIT)));

    // Create and configure the compiler
    let mut compiler_config = wasmer::Cranelift::default();
    let gas_middleware = GasMiddleware::new(None);
    compiler_config.push_middleware(Arc::new(gas_middleware));

    // Create an store
    let engine = wasmer::sys::EngineBuilder::new(compiler_config).engine();
    let mut store = Store::new(&engine);

    let bytecode = match wasmer::wat2wasm(code) {
        Ok(m) => m,
        Err(e) => {
            return Err(anyhow::Error::msg(e.to_string()));
        }
    };

    let module = match Module::new(&store, bytecode.clone()) {
        Ok(m) => m,
        Err(e) => {
            debug!("create_wasm_instance->new_module_error:{:?}", &e);
            return Err(anyhow::Error::msg(e.to_string()));
        }
    };

    let env = FunctionEnv::new(
        &mut store,
        Env {
            memory: None,
            gas_meter: gas_meter.clone(),
        },
    );

    let import_object = imports! {
        "wasi_snapshot_preview1" => {
            "fd_write" => Function::new_typed_with_env(&mut store, &env, fd_write),
            "fd_seek" => Function::new_typed_with_env(&mut store, &env, fd_seek),
            "fd_close" => Function::new_typed_with_env(&mut store, &env, fd_close),
            "proc_exit" => Function::new_typed_with_env(&mut store, &env, proc_exit),
        },
        "env" => {
            "js_log" => Function::new_typed_with_env(&mut store, &env, js_log),
            "charge" => Function::new_typed_with_env(&mut store, &env, charge),
        },
    };

    let instance = match Instance::new(&mut store, &module, &import_object) {
        Ok(v) => v,
        Err(e) => {
            debug!("create_wasm_instance->new_instance_error:{:?}", &e);
            return Err(anyhow::Error::msg("create wasm instance failed"));
        }
    };

    if let Ok(memory) = instance.exports.get_memory("memory") {
        env.as_mut(&mut store).memory = Some(Arc::new(Mutex::new(memory.clone())));
    }

    Ok(WASMInstance::new(
        bytecode.to_vec(),
        instance,
        store,
        gas_meter,
    ))
}
