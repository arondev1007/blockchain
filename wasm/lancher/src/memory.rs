use borsh::{BorshDeserialize, BorshSerialize};
use wasmer::{Instance, MemoryView, Store, StoreMut};

use memory::Memory;

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize, Clone)]
pub enum EmMemError {
    MemoryWriteFail(String),
    MemoryWriteLoadFail(String),

    MemoryReadDataLenFail(String),
    MemoryReadDataFail(String),
    MemoryReadGetMemoryFail(String),

    MemoryAllocGetFnFail(String),
    MemoryAllocCallFnFail(String),
    MemoryAllocPtrEmpty,
}

pub struct VmMemory;

impl VmMemory {
    pub fn mem_write_store(
        store: &mut Store,
        instance: &Instance,
        data: &[u8],
    ) -> Result<u32, EmMemError> {
        // alloc - memory
        let ptr = VmMemory::mem_alloc_store(store, instance, data.len() as u32)?;

        // load - memory
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|e| EmMemError::MemoryWriteLoadFail(e.to_string()))?;

        // load - memory view
        let memory_view = memory.view(store);
        VmMemory::mem_write(memory_view, ptr, data)
    }

    pub fn mem_write_mut_store(
        store: &mut StoreMut,
        instance: &Instance,
        val: &[u8],
    ) -> Result<u32, EmMemError> {
        // alloc - memory
        let ptr = VmMemory::mem_alloc_store_mut(store, instance, (val.len() as u32) + 4)?;

        // load - memory
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|e| EmMemError::MemoryWriteLoadFail(e.to_string()))?;

        // load - memory view
        let memory_view = memory.view(store);
        VmMemory::mem_write(memory_view, ptr, val)
    }

    pub fn mem_read_store(
        store: &mut Store,
        instance: &Instance,
        ptr: u32,
    ) -> Result<Vec<u8>, EmMemError> {
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|e| EmMemError::MemoryReadGetMemoryFail(e.to_string()))?;

        let memory_view = memory.view(store);
        VmMemory::mem_read(&memory_view, ptr)
    }

    pub fn mem_write(memory_view: MemoryView, ptr: u32, data: &[u8]) -> Result<u32, EmMemError> {
        // encode - data ( len (4byte)  + data )
        let buffer = Memory::encode(data);

        // write - memory
        memory_view
            .write(ptr as u64, &buffer)
            .map_err(|e| EmMemError::MemoryWriteFail(e.to_string()))?;

        Ok(ptr)
    }

    pub fn mem_read(mem_view: &MemoryView, ptr: u32) -> Result<Vec<u8>, EmMemError> {
        // read - memory ( data len )
        let mut buffer = vec![0; 4];
        mem_view
            .read(ptr as u64, &mut buffer)
            .map_err(|e| EmMemError::MemoryReadDataLenFail(e.to_string()))?;

        let len = Memory::decode_len(&buffer);

        // init - buffer
        let mut buffer = vec![0; len];

        // read - memory ( data )
        mem_view
            .read((ptr as u64) + 4, &mut buffer)
            .map_err(|e| EmMemError::MemoryReadDataFail(e.to_string()))?;

        Ok(buffer)
    }

    pub fn mem_alloc_store(
        store: &mut Store,
        instance: &Instance,
        size: u32,
    ) -> Result<u32, EmMemError> {
        // load - function
        let mem_alloc_fn = instance
            .exports
            .get_function("mem_alloc")
            .map_err(|e| EmMemError::MemoryAllocGetFnFail(e.to_string()))?;

        // call - function
        let fn_result = mem_alloc_fn
            .call(store, &[size.into()])
            .map_err(|e| EmMemError::MemoryAllocCallFnFail(e.to_string()))?;

        // load - ptr
        let ptr = fn_result[0].i32().ok_or(EmMemError::MemoryAllocPtrEmpty)?;
        Ok(ptr as u32)
    }

    pub fn mem_alloc_store_mut(
        store: &mut StoreMut,
        instance: &Instance,
        size: u32,
    ) -> Result<u32, EmMemError> {
        // load - function
        let mem_alloc_fn = instance
            .exports
            .get_function("mem_alloc")
            .map_err(|e| EmMemError::MemoryAllocGetFnFail(e.to_string()))?;

        // call - function
        let fn_result = mem_alloc_fn
            .call(store, &[size.into()])
            .map_err(|e| EmMemError::MemoryAllocCallFnFail(e.to_string()))?;

        // load - ptr
        let ptr = fn_result[0].i32().ok_or(EmMemError::MemoryAllocPtrEmpty)?;
        Ok(ptr as u32)
    }
}
