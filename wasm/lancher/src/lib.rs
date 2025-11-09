pub mod core;
pub mod data;
pub mod memory;

use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::u64;

pub use wasmer::*;
use wasmer_middlewares::metering::set_remaining_points;
pub use wasmparser::Operator;

use crate::core::gas::*;
use crate::core::instance::*;
use crate::core::module::*;
use crate::data::*;
use crate::memory::*;

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize, Clone)]
pub enum EmVmError {
    // WasmFunction
    FunctionExportFail(String),
    FunctionCallFail(String),
    FunctionCallOutOfGas,

    // Initialize
    NewOpcodeBinaryEmpty,
    NewModuleInitBinaryFail(ModuleError),
    NewModuleInitEncodedFail(ModuleError),
    NewInstanceInitFail(InstanceError),
    ExportModuleFail(ModuleError),
    RetProgramMemReadFail(EmMemError),
}

pub type GasConsumptionFn = Arc<dyn Fn(&Operator) -> u64 + Send + Sync + 'static>;

pub struct VMLauncher<T: Send + Sync + Clone + 'static> {
    vm_module: VmModule,
    store: Store,
    instance: Instance,
    gas_used: bool,

    #[allow(dead_code)]
    external: Option<T>,
}

impl VMLauncher<()> {
    pub fn new(
        opcode: &[u8],
        opcode_module_used: bool, // module 압축된 opcode 사용 여부
        gas_metering_used: bool,
        gas_consumption: Option<GasConsumptionFn>,
    ) -> Result<Self, EmVmError> {
        // check - opcode binary
        if opcode.is_empty() {
            return Err(EmVmError::NewOpcodeBinaryEmpty);
        }

        // init - gas
        let mut store: Store;
        let gas_used: bool;
        match gas_metering_used {
            true => {
                store = Store::new(EngineBuilder::new(GasMetering::create_cfg(gas_consumption)));
                gas_used = true;
            }
            false => {
                store = Store::default();
                gas_used = false;
            }
        }

        // init - module
        let mut vm_module = VmModule::new();
        match opcode_module_used {
            true => {
                vm_module
                    .import_module_opcode(&store, opcode)
                    .map_err(|e| EmVmError::NewModuleInitEncodedFail(e))?;
            }
            false => {
                vm_module
                    .import(&mut store, opcode)
                    .map_err(|e| EmVmError::NewModuleInitBinaryFail(e))?;
            }
        }

        // init - instance
        let instance = VmInstance::new::<ImportedFn<()>>(
            &mut store,
            vm_module.borrow(),
            VmData::new(),
            None::<()>,
            HashMap::new(),
        )
        .map_err(|e| EmVmError::NewInstanceInitFail(e))?
        .unwrap();

        Ok(VMLauncher {
            vm_module,
            store,
            instance,
            gas_used,
            external: None,
        })
    }
}

impl<T: Send + Sync + Clone + 'static> VMLauncher<T> {
    pub const DEF_PROGRAM_RET_EMPTY: Vec<u8> = Vec::new();

    pub fn new_with_external(
        opcode: &[u8],
        opcode_module_used: bool, // module 압축된 opcode 사용 여부
        gas_metering_used: bool,
        external: T,
        imported_fn: HashMap<String, (ImportedFn<T>, FunctionType)>,
        gas_consumption: Option<GasConsumptionFn>,
    ) -> Result<Self, EmVmError> {
        // check - opcode binary
        if opcode.is_empty() {
            return Err(EmVmError::NewOpcodeBinaryEmpty);
        }

        // init - gas
        let mut store: Store;
        let gas_used: bool;
        match gas_metering_used {
            true => {
                store = Store::new(EngineBuilder::new(GasMetering::create_cfg(gas_consumption)));
                gas_used = true;
            }
            false => {
                store = Store::default();
                gas_used = false;
            }
        }

        // init - module
        let mut vm_module = VmModule::new();
        match opcode_module_used {
            true => {
                vm_module
                    .import_module_opcode(&store, opcode)
                    .map_err(|e| EmVmError::NewModuleInitEncodedFail(e))?;
            }
            false => {
                vm_module
                    .import(&mut store, opcode)
                    .map_err(|e| EmVmError::NewModuleInitBinaryFail(e))?;
            }
        }

        // init - instance
        let instance = VmInstance::new(
            &mut store,
            vm_module.borrow(),
            VmData::new(),
            Some(external.clone()),
            imported_fn,
        )
        .map_err(|e| EmVmError::NewInstanceInitFail(e))?
        .unwrap();

        Ok(VMLauncher {
            vm_module,
            store,
            instance,
            gas_used,
            external: Some(external),
        })
    }

    pub fn run(&mut self, gas_priority: u64, gas_limit: u64, fn_name: &str) -> VmRunResult {
        // set - gas limit
        let mut gas_limit_calc = 0;
        if gas_priority != 0 {
            gas_limit_calc = self.calc_gas(gas_priority, gas_limit);
            set_remaining_points(&mut self.store, &self.instance, gas_limit_calc);
        }

        // export - wasm fn
        let ret_fn = self.instance.exports.get_function(fn_name);
        if let Err(e) = ret_fn {
            return VmRunResult::new(
                Some(EmVmError::FunctionExportFail(format!("{:?}", e))),
                ProgramCode::FnInvalidEntryPoint,
                Self::DEF_PROGRAM_RET_EMPTY,
                0,
            );
        }

        // call - wasm fn
        let ret_box_value = ret_fn.unwrap().call(&mut self.store, &[]);
        if let Err(e) = ret_box_value {
            let u64_gas_left = self.get_gas_left();
            match u64_gas_left {
                0 => {
                    return VmRunResult::new(
                        Some(EmVmError::FunctionCallOutOfGas),
                        ProgramCode::OutOfGas,
                        Self::DEF_PROGRAM_RET_EMPTY,
                        gas_limit, // 모든 가스 소진하여 입력된 가스 총량 리턴
                    );
                }
                _ => {
                    return VmRunResult::new(
                        Some(EmVmError::FunctionCallFail(format!("{:?}", e))),
                        ProgramCode::UnknownError,
                        Self::DEF_PROGRAM_RET_EMPTY,
                        gas_limit_calc - u64_gas_left,
                    );
                }
            }
        }

        // get - gas left
        let gas_left = self.get_gas_left();

        // return - program result
        // wasm module 사용을 위해 항상 진입 가스 priority 를 고정값 ( 0 ) 을 넣음으로
        // 최종 가스 소모량을 계산할때 priority 를 곱해줘야 한다.
        self.ret_program(
            ret_box_value.unwrap(),
            (gas_limit_calc - gas_left) * gas_priority,
        )
    }

    pub fn get_module_opcode(&mut self) -> Result<Vec<u8>, EmVmError> {
        let module_bytes = self
            .vm_module
            .export_module_opcode()
            .map_err(|e| EmVmError::ExportModuleFail(e))?;

        Ok(module_bytes)
    }

    fn get_gas_left(&mut self) -> u64 {
        match self.gas_used {
            true => GasMetering::get_left(&mut self.store, &self.instance),
            false => return 0,
        }
    }

    fn calc_gas(&self, gas_priority: u64, gas_limit: u64) -> u64 {
        gas_limit / gas_priority
    }

    fn ret_program(&mut self, value: Box<[Value]>, gas_used: u64) -> VmRunResult {
        // check - empty
        if value.is_empty() {
            return VmRunResult::new(
                None,
                ProgramCode::UnknownError,
                Self::DEF_PROGRAM_RET_EMPTY,
                gas_used,
            );
        }

        // load - ptr
        let ptr = match value[0].i32() {
            Some(ptr) => ptr as u32,
            None => {
                return VmRunResult::new(
                    None,
                    ProgramCode::UndefinedErrPtr,
                    Self::DEF_PROGRAM_RET_EMPTY,
                    gas_used,
                );
            }
        };

        // read - memory ( in wasm )
        let result = match VmMemory::mem_read_store(&mut self.store, &self.instance, ptr) {
            Ok(result) => result,
            Err(e) => {
                return VmRunResult::new(
                    Some(EmVmError::RetProgramMemReadFail(e)),
                    ProgramCode::UndefinedErrPtr,
                    Self::DEF_PROGRAM_RET_EMPTY,
                    gas_used,
                );
            }
        };

        // load - program ret type
        let program_err = ProgramCode::from_arr_u8(&result[0..1]);
        match program_err {
            // proc - code ok
            ProgramCode::Ok => {
                let fn_ret_data = result[1..].to_vec();
                VmRunResult::new(None, ProgramCode::Ok, fn_ret_data, gas_used)
            }

            // proc - code error & abort
            _ => {
                let program_ret_code_bytes = result;
                let program_ret_code = ProgramCode::from_arr_u8(&program_ret_code_bytes);
                VmRunResult::new(
                    None,
                    program_ret_code,
                    Self::DEF_PROGRAM_RET_EMPTY,
                    gas_used,
                )
            }
        }
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum ProgramCode {
    Ok,
    FnInvalidEntryPoint,
    FnInvalidIndex,
    FnInvalidArgs,

    UndefinedErrPtr,
    UnknownError,

    OutOfGas,
    VmError,

    BorshEncodeInvalidArg,
    BorshDecodeInvalidArg,
}

impl ProgramCode {
    pub fn from_arr_u8(err: &[u8]) -> Self {
        match err {
            x if x == ProgramCode::Ok.to_vec_u8() => ProgramCode::Ok,
            x if x == ProgramCode::FnInvalidEntryPoint.to_vec_u8() => {
                ProgramCode::FnInvalidEntryPoint
            }
            x if x == ProgramCode::FnInvalidIndex.to_vec_u8() => ProgramCode::FnInvalidIndex,
            x if x == ProgramCode::FnInvalidArgs.to_vec_u8() => ProgramCode::FnInvalidArgs,
            x if x == ProgramCode::UnknownError.to_vec_u8() => ProgramCode::UnknownError,
            x if x == ProgramCode::UndefinedErrPtr.to_vec_u8() => ProgramCode::UndefinedErrPtr,
            x if x == ProgramCode::OutOfGas.to_vec_u8() => ProgramCode::OutOfGas,
            x if x == ProgramCode::VmError.to_vec_u8() => ProgramCode::VmError,
            x if x == ProgramCode::BorshEncodeInvalidArg.to_vec_u8() => {
                ProgramCode::BorshEncodeInvalidArg
            }
            x if x == ProgramCode::BorshDecodeInvalidArg.to_vec_u8() => {
                ProgramCode::BorshDecodeInvalidArg
            }
            _ => ProgramCode::UnknownError,
        }
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        match self {
            ProgramCode::Ok => vec![ProgramCode::Ok.to_i32() as u8],
            ProgramCode::FnInvalidEntryPoint => {
                vec![ProgramCode::FnInvalidEntryPoint.to_i32() as u8]
            }
            ProgramCode::FnInvalidIndex => vec![ProgramCode::FnInvalidIndex.to_i32() as u8],
            ProgramCode::FnInvalidArgs => vec![ProgramCode::FnInvalidArgs.to_i32() as u8],
            ProgramCode::UnknownError => vec![ProgramCode::UnknownError.to_i32() as u8],
            ProgramCode::UndefinedErrPtr => vec![ProgramCode::UndefinedErrPtr.to_i32() as u8],
            ProgramCode::OutOfGas => vec![ProgramCode::OutOfGas.to_i32() as u8],
            ProgramCode::VmError => vec![ProgramCode::VmError.to_i32() as u8],
            ProgramCode::BorshEncodeInvalidArg => {
                vec![ProgramCode::BorshEncodeInvalidArg.to_i32() as u8]
            }
            ProgramCode::BorshDecodeInvalidArg => {
                vec![ProgramCode::BorshDecodeInvalidArg.to_i32() as u8]
            }
        }
    }

    pub fn from_i32(err: i32) -> Self {
        match err {
            x if x == ProgramCode::Ok.to_i32() => ProgramCode::Ok,
            x if x == ProgramCode::FnInvalidEntryPoint.to_i32() => ProgramCode::FnInvalidEntryPoint,
            x if x == ProgramCode::FnInvalidIndex.to_i32() => ProgramCode::FnInvalidIndex,
            x if x == ProgramCode::FnInvalidArgs.to_i32() => ProgramCode::FnInvalidArgs,
            x if x == ProgramCode::UnknownError.to_i32() => ProgramCode::UnknownError,
            x if x == ProgramCode::UndefinedErrPtr.to_i32() => ProgramCode::UndefinedErrPtr,
            x if x == ProgramCode::OutOfGas.to_i32() => ProgramCode::OutOfGas,
            x if x == ProgramCode::VmError.to_i32() => ProgramCode::VmError,
            x if x == ProgramCode::BorshEncodeInvalidArg.to_i32() => {
                ProgramCode::BorshEncodeInvalidArg
            }
            x if x == ProgramCode::BorshDecodeInvalidArg.to_i32() => {
                ProgramCode::BorshDecodeInvalidArg
            }
            _ => ProgramCode::UnknownError,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            ProgramCode::Ok => ProgramCode::Ok as i32,
            ProgramCode::FnInvalidEntryPoint => ProgramCode::FnInvalidEntryPoint as i32,
            ProgramCode::FnInvalidIndex => ProgramCode::FnInvalidIndex as i32,
            ProgramCode::FnInvalidArgs => ProgramCode::FnInvalidArgs as i32,
            ProgramCode::UnknownError => ProgramCode::UnknownError as i32,
            ProgramCode::UndefinedErrPtr => ProgramCode::UndefinedErrPtr as i32,
            ProgramCode::OutOfGas => ProgramCode::OutOfGas as i32,
            ProgramCode::VmError => ProgramCode::VmError as i32,
            ProgramCode::BorshEncodeInvalidArg => ProgramCode::BorshEncodeInvalidArg as i32,
            ProgramCode::BorshDecodeInvalidArg => ProgramCode::BorshDecodeInvalidArg as i32,
        }
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct VmRunResult {
    pub error: Option<EmVmError>,
    pub program_code: ProgramCode,
    pub program_data: Vec<u8>,
    pub gas_used: u64,
}

impl VmRunResult {
    pub fn new(
        err: Option<EmVmError>,
        program_code: ProgramCode,
        program_data: Vec<u8>,
        gas_used: u64,
    ) -> Self {
        VmRunResult {
            error: err,
            program_code,
            program_data,
            gas_used,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::{fs, sync::Arc};

    const FILE_PATH_WASM: &str = "main.wasm";

    #[test]
    fn run_basic() {
        let wasm_binary = load_file(FILE_PATH_WASM);
        let is_module = false;
        let fn_name = "example";
        let gas_priority = 0;
        let gas_limit = 0;

        // init
        let vm_launcher = VMLauncher::new(
            &wasm_binary,
            is_module,
            false,
            Some(custom_gas_consumption()),
        );
        assert!(vm_launcher.is_ok(), "{:?}", vm_launcher.err());

        // run vm
        let vm_ret = vm_launcher.unwrap().run(gas_priority, gas_limit, fn_name);
        println!("result : {:?}", vm_ret);
    }

    #[test]
    fn run_basic_with_gas() {
        let opcode = load_file(FILE_PATH_WASM);
        let is_module = false;
        let gas_priority = 1;
        let gas_limit = 10000000;
        let fn_name = "example";

        // init
        let launcher = VMLauncher::new(&opcode, is_module, true, Some(custom_gas_consumption()));
        assert!(launcher.is_ok(), "{:?}", launcher.err());

        // run launcher
        let vm_ret = launcher.unwrap().run(gas_priority, gas_limit, fn_name);
        println!("result : {:?}", vm_ret);
    }

    #[test]
    fn run_module_with_gas() {
        // 모듈을 만들기 위해 생성한 인스턴스에 입력한 gas_price 와
        // 모듈을 실행하기 위해 생성한 인스턴스에 입력할 gas_price 는
        // 반드시 같아야 한다.
        let opcode = get_opcode_type_module();
        let is_module = true;
        let priority = 1;
        let limit = 10000000;
        let fn_name = "example";

        assert!(opcode.is_ok(), "{:?}", opcode.err());

        // init
        let vm_launcher = VMLauncher::new(
            &opcode.unwrap(),
            is_module,
            true,
            Some(custom_gas_consumption()),
        );
        assert!(vm_launcher.is_ok(), "{:?}", vm_launcher.err());

        // run vm
        let result = vm_launcher.unwrap().run(priority, limit, fn_name);
        println!("result : {:?}", result);
    }

    fn get_opcode_type_module() -> Result<Vec<u8>, EmVmError> {
        let opcode = load_file(FILE_PATH_WASM);
        let is_module = false;

        // init launcher
        let vm_launcher = VMLauncher::new(&opcode, is_module, true, Some(custom_gas_consumption()));
        if vm_launcher.is_err() {
            return Err(vm_launcher.err().unwrap());
        }

        let module_opcode = vm_launcher.unwrap().get_module_opcode();
        if module_opcode.is_err() {
            return Err(module_opcode.err().unwrap());
        }

        Ok(module_opcode.unwrap())
    }

    fn load_file(file_path: &str) -> Vec<u8> {
        let wasm_bytes = fs::read(file_path).expect("Failed to read file");
        return wasm_bytes;
    }

    fn custom_gas_consumption() -> Arc<dyn Fn(&Operator) -> u64 + Send + Sync + 'static> {
        Arc::new(move |operator: &Operator| -> u64 {
            let gas_by_opcode = match operator {
                Operator::BrTable { .. } => 120,
                Operator::Return { .. } => 90,

                Operator::Call { .. } => 90,
                Operator::CallIndirect { .. } => 10000,

                Operator::I32Const { .. } => 1,
                Operator::I32Add { .. } => 45,
                Operator::I32Sub { .. } => 45,
                Operator::I32Mul { .. } => 45,
                Operator::I32DivS { .. } => 36000,
                Operator::I32DivU { .. } => 36000,
                Operator::I32RemS { .. } => 36000,
                Operator::I32RemU { .. } => 36000,
                Operator::I32And { .. } => 45,
                Operator::I32Or { .. } => 45,
                Operator::I32Xor { .. } => 45,
                Operator::I32Shl { .. } => 67,
                Operator::I32ShrU { .. } => 67,
                Operator::I32ShrS { .. } => 67,
                Operator::I32Rotl { .. } => 90,
                Operator::I32Rotr { .. } => 90,
                Operator::I32Eq { .. } => 45,
                Operator::I32Eqz { .. } => 45,
                Operator::I32Ne { .. } => 45,
                Operator::I32LtS { .. } => 45,
                Operator::I32LtU { .. } => 45,
                Operator::I32LeS { .. } => 45,
                Operator::I32LeU { .. } => 45,
                Operator::I32GtS { .. } => 45,
                Operator::I32GtU { .. } => 45,
                Operator::I32GeS { .. } => 45,
                Operator::I32GeU { .. } => 45,
                Operator::I32Clz { .. } => 45,
                Operator::I32Ctz { .. } => 45,
                Operator::I32Popcnt { .. } => 45,

                Operator::Drop { .. } => 120,
                Operator::Select { .. } => 120,
                Operator::Unreachable { .. } => 1,
                _ => 1,
            };
            gas_by_opcode * 10
        })
    }
}
