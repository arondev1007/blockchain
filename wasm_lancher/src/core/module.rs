use borsh::{BorshDeserialize, BorshSerialize};
use wasmer::{Module, Store};

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize, Clone)]
pub enum ModuleError {
    InitByWasmBinaryFail(String),
    InitByEncodedModuleFail(String),

    ExportFileModuleEmpty,

    ExportVecModuleEmpty,
    ExportVecModuleSerializeFail(String),
}

#[derive(Debug)]
pub struct VmModule {
    op_module: Option<Module>,
}

impl VmModule {
    pub fn new() -> Self {
        VmModule { op_module: None }
    }

    pub fn import(&mut self, store: &Store, wasm_binary: &[u8]) -> Result<(), ModuleError> {
        // new - module
        let module = Module::new(store, wasm_binary)
            .map_err(|e| ModuleError::InitByWasmBinaryFail(e.to_string()))?;

        // save
        self.op_module = Some(module);
        Ok(())
    }

    pub fn import_module_opcode(
        &mut self,
        mut store: &Store,
        encoded_module: &[u8],
    ) -> Result<(), ModuleError> {
        // deserialize - encoded module
        let module = unsafe { Module::deserialize(&mut store, encoded_module) }
            .map_err(|e| ModuleError::InitByEncodedModuleFail(e.to_string()))?;

        // save
        self.op_module = Some(module);
        Ok(())
    }

    pub fn borrow(&mut self) -> &wasmer::Module {
        self.op_module.as_ref().unwrap()
    }

    pub fn export_module_opcode(&self) -> Result<Vec<u8>, ModuleError> {
        let module = self
            .op_module
            .clone()
            .ok_or(ModuleError::ExportVecModuleEmpty)?;

        // serialize - module
        let module_bytes = module
            .serialize()
            .map_err(|e| ModuleError::ExportVecModuleSerializeFail(e.to_string()))?;

        Ok(module_bytes.to_vec())
    }
}
