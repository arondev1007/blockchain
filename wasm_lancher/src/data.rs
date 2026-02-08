use wasmer::{AsStoreMut, Instance, Memory, MemoryView, StoreMut};

use crate::memory::*;

type Ptr = u32;
pub const DEF_PTR_ERR: i32 = 0;

#[derive(Debug, PartialEq)]
pub enum VmDataError {
    MemoryWriteInstanceEmpty,
    MemoryWriteFail(EmMemError),

    MemoryReadViewEmpty,
    MemoryReadFail(EmMemError),
}

pub struct VmData {
    pub instance: Option<Instance>,
    pub memory: Option<Memory>,
}

impl Clone for VmData {
    fn clone(&self) -> Self {
        VmData {
            instance: None,
            memory: None,
        }
    }
}

impl VmData {
    pub fn new() -> Self {
        VmData {
            instance: None,
            memory: None,
        }
    }

    pub fn init(&mut self) {}

    pub fn instance_set(&mut self, instance: Instance) {
        self.instance = Some(instance);
    }

    pub fn instance_get(&self) -> Option<&Instance> {
        self.instance.as_ref()
    }

    pub fn memory_set(&mut self, memory: &Memory) {
        self.memory = Some(memory.clone());
    }

    pub fn memory_get<'a>(&self, store: &'a impl AsStoreMut) -> Option<MemoryView<'a>> {
        let result = self.memory.as_ref();
        if result.is_none() {
            return None;
        }

        let memory_view = result.unwrap().view(store);
        Some(memory_view)
    }

    pub fn memory_write(&mut self, store: &mut StoreMut, data: &[u8]) -> Result<Ptr, VmDataError> {
        // load - instance
        let instance = self
            .instance_get()
            .ok_or(VmDataError::MemoryWriteInstanceEmpty)?;

        // write - memory
        let ptr = VmMemory::mem_write_mut_store(store, instance, data)
            .map_err(|e| VmDataError::MemoryWriteFail(e))?;

        Ok(ptr)
    }

    pub fn memory_read(
        &mut self,
        store: &mut StoreMut,
        ptr: Vec<i32>,
    ) -> Result<Vec<Vec<u8>>, VmDataError> {
        let mut memory_read: Vec<Vec<u8>> = vec![];

        // load - memory view
        let memory_view = self
            .memory_get(store)
            .ok_or(VmDataError::MemoryReadViewEmpty)?;

        // read - memory
        for ptr in ptr {
            let vec_u8 = VmMemory::mem_read(&memory_view, ptr as u32)
                .map_err(|e| VmDataError::MemoryReadFail(e))?;
            memory_read.push(vec_u8);
        }

        Ok(memory_read)
    }
}
