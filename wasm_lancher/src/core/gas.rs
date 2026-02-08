use std::sync::Arc;
pub use wasmer::wasmparser::Operator;
use wasmer::{CompilerConfig, Cranelift, Instance, Store, StoreMut};
use wasmer_middlewares::{
    Metering,
    metering::{MeteringPoints, get_remaining_points, set_remaining_points},
};

#[derive(Debug)]
pub struct GasMetering;

impl GasMetering {
    pub const DEF_GAS_PRIORITY: u64 = 1;

    pub fn create_cfg(
        gas_consumption: Option<Arc<dyn Fn(&Operator) -> u64 + Send + Sync + 'static>>,
    ) -> impl CompilerConfig {
        // Set gas limit to 0 for module replication
        // Once module creation is complete, gas is injected.
        let gas_limit = 0;

        // Determine gas consumption: use provided or default
        let arc_fn = gas_consumption.unwrap_or_else(|| Self::set_default_consumption());
        let consumption_fn = move |operator: &Operator| -> u64 { arc_fn(operator) };
        let metering = Arc::new(Metering::new(gas_limit, consumption_fn));

        // Set compiler config with the metering middleware
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(metering);

        compiler_config
    }

    pub fn get_left(store: &mut Store, instance: &Instance) -> u64 {
        let gas_left: u64 = match get_remaining_points(store, instance) {
            MeteringPoints::Remaining(points) => points,
            MeteringPoints::Exhausted => 0,
        };

        gas_left
    }

    pub fn gas_decrease(store: &mut StoreMut<'_>, instance: &Instance, gas_expected: u64) -> bool {
        let gas_left = GasMetering::get_left_store_mute(store, instance);
        if gas_left < gas_expected {
            return false;
        }

        GasMetering::set_store_mute(store, instance, gas_left - gas_expected);
        true
    }

    pub fn get_left_store_mute<'a>(store: &mut StoreMut, instance: &Instance) -> u64 {
        // load - gas left
        let gas_left: u64 = match get_remaining_points(store, instance) {
            MeteringPoints::Remaining(points) => points,
            MeteringPoints::Exhausted => 0,
        };

        gas_left
    }

    pub fn set_store_mute(store: &mut StoreMut, instance: &Instance, u64_gas: u64) {
        set_remaining_points(store, instance, u64_gas);
    }

    fn set_default_consumption() -> Arc<dyn Fn(&Operator) -> u64 + Send + Sync + 'static> {
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
            gas_by_opcode * Self::DEF_GAS_PRIORITY
        })
    }
}
