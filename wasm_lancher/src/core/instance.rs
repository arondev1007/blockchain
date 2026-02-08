use std::{collections::HashMap, marker::PhantomData};

use borsh::{BorshDeserialize, BorshSerialize};
use wasmer::{
    Exports, Function, FunctionEnv, FunctionEnvMut, FunctionType, Imports, Instance, Module,
    RuntimeError, Store, Value,
};

use crate::data::*;

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize, Clone)]
pub enum InstanceError {
    NewInstanceCreateFail(String),
}

pub struct VmInstance<T: Send + Sync + Clone + 'static> {
    _marker: PhantomData<T>,
}

pub type ImportedFn<T> = Box<
    dyn (Fn(FunctionEnvMut<'_, (VmData, Option<T>)>, &[Value]) -> Result<Vec<Value>, RuntimeError>)
        + Send
        + Sync
        + 'static,
>;

impl<T: Send + Sync + Clone + 'static> VmInstance<T> {
    pub fn new<F>(
        store: &mut Store,
        module: &Module,
        vm_data: VmData,
        external: Option<T>,
        imported_fn: HashMap<String, (F, FunctionType)>,
    ) -> Result<Option<Instance>, InstanceError>
    where
        F: Fn(
                FunctionEnvMut<'_, (VmData, Option<T>)>,
                &[Value],
            ) -> Result<Vec<Value>, RuntimeError>
            + Send
            + Sync
            + 'static,
    {
        // init - imports & env
        let mut import_obj = Imports::new();
        let mut vm_env_imports = Exports::new();
        let vm_env = FunctionEnv::new(store, (vm_data, external));

        // set - imports & env
        for (fn_name, (fn_instance, fn_type)) in imported_fn {
            vm_env_imports.insert(
                fn_name,
                Function::new_with_env(store, &vm_env, fn_type, fn_instance),
            );
        }

        import_obj.register_namespace("env", vm_env_imports);

        // new - instance
        let instance = Instance::new(store, module, &import_obj)
            .map_err(|e| InstanceError::NewInstanceCreateFail(e.to_string()))?;

        // load - env mut
        let mut vm_env_mut = vm_env.into_mut(store);
        let (vm_data, _opt_external) = vm_env_mut.data_mut();

        // save - instance & memory
        vm_data.instance_set(instance.clone());
        let memory = instance.exports.get_memory("memory").unwrap();
        vm_data.memory_set(memory);

        Ok(Some(instance))
    }
}
