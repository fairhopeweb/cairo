use bimap::BiMap;
use num_bigint::BigInt;
use test_case::test_case;

use super::value::CoreValue::{
    self, Array, GasBuiltin, NonZero, RangeCheck, Uint128, Uninitialized,
};
use super::LibFuncSimulationError::{
    self, FunctionSimulationError, MemoryLayoutMismatch, WrongNumberOfArgs,
};
use super::{core, SimulationError};
use crate::extensions::core::CoreLibFunc;
use crate::extensions::lib_func::{
    SierraApChange, SignatureSpecializationContext, SpecializationContext,
};
use crate::extensions::type_specialization_context::TypeSpecializationContext;
use crate::extensions::types::TypeInfo;
use crate::extensions::GenericLibFunc;
use crate::ids::{ConcreteTypeId, FunctionId, GenericTypeId};
use crate::program::{ConcreteTypeLongId, Function, FunctionSignature, GenericArg, StatementIdx};
use crate::test_utils::build_bijective_mapping;

fn type_arg(name: &str) -> GenericArg {
    GenericArg::Type(name.into())
}

fn value_arg(v: i64) -> GenericArg {
    GenericArg::Value(BigInt::from(v))
}

fn user_func_arg(name: &str) -> GenericArg {
    GenericArg::UserFunc(name.into())
}

struct MockSpecializationContext {
    mapping: BiMap<ConcreteTypeId, ConcreteTypeLongId>,
}
impl MockSpecializationContext {
    pub fn new() -> Self {
        Self { mapping: build_bijective_mapping() }
    }
}

impl SpecializationContext for MockSpecializationContext {
    fn upcast(&self) -> &dyn SignatureSpecializationContext {
        self
    }

    fn try_get_function(&self, function_id: &FunctionId) -> Option<Function> {
        ["drop_all_inputs", "identity", "unimplemented"]
            .into_iter()
            .map(|name| -> FunctionId { name.into() })
            .find(|id: &FunctionId| id == function_id)
            .map(|_| Function::new(function_id.clone(), vec![], vec![], StatementIdx(0)))
    }
}
impl TypeSpecializationContext for MockSpecializationContext {
    fn try_get_type_info(&self, id: ConcreteTypeId) -> Option<TypeInfo> {
        if id == "uint128".into() || id == "NonZeroInt".into() {
            Some(TypeInfo {
                long_id: self.mapping.get_by_left(&id)?.clone(),
                storable: true,
                droppable: true,
                duplicatable: true,
                size: 1,
            })
        } else if id == "UninitializedInt".into() {
            Some(TypeInfo {
                long_id: self.mapping.get_by_left(&id)?.clone(),
                storable: false,
                droppable: true,
                duplicatable: false,
                size: 0,
            })
        } else {
            None
        }
    }
}
impl SignatureSpecializationContext for MockSpecializationContext {
    fn try_get_concrete_type(
        &self,
        id: GenericTypeId,
        generic_args: &[GenericArg],
    ) -> Option<ConcreteTypeId> {
        self.mapping
            .get_by_right(&ConcreteTypeLongId {
                generic_id: id,
                generic_args: generic_args.to_vec(),
            })
            .cloned()
    }

    fn try_get_function_signature(&self, function_id: &FunctionId) -> Option<FunctionSignature> {
        self.try_get_function(function_id).map(|f| f.signature)
    }

    fn as_type_specialization_context(&self) -> &dyn TypeSpecializationContext {
        self
    }

    fn try_get_function_ap_change(&self, _function_id: &FunctionId) -> Option<SierraApChange> {
        Some(SierraApChange::NotImplemented)
    }
}

/// Expects to find a libfunc and simulate it.
fn simulate(
    id: &str,
    generic_args: Vec<GenericArg>,
    inputs: Vec<CoreValue>,
) -> Result<(Vec<CoreValue>, usize), LibFuncSimulationError> {
    core::simulate(
        &CoreLibFunc::by_id(&id.into())
            .unwrap()
            .specialize(&MockSpecializationContext::new(), &generic_args)
            .unwrap(),
        inputs,
        || Some(4),
        |id, inputs| {
            if id == &"drop_all_inputs".into() {
                Ok(vec![])
            } else if id == &"identity".into() {
                Ok(inputs)
            } else {
                Err(FunctionSimulationError(
                    id.clone(),
                    Box::new(SimulationError::StatementOutOfBounds(StatementIdx(0))),
                ))
            }
        },
    )
}

#[test_case("get_gas", vec![], vec![RangeCheck, GasBuiltin(5)]
             => Ok((vec![RangeCheck, GasBuiltin(1)], 0)); "get_gas(5)")]
#[test_case("get_gas", vec![], vec![RangeCheck, GasBuiltin(2)]
             => Ok((vec![RangeCheck, GasBuiltin(2)], 1)); "get_gas(2)")]
#[test_case("uint128_jump_nz", vec![], vec![Uint128(2)]
             => Ok((vec![NonZero(Box::new(Uint128(2)))], 1)); "uint128_jump_nz(2)")]
#[test_case("uint128_jump_nz", vec![], vec![Uint128(0)] => Ok((vec![], 0)); "uint128_jump_nz(0)")]
#[test_case("jump", vec![], vec![] => Ok((vec![], 0)); "jump()")]
#[test_case("uint128_checked_add", vec![], vec![RangeCheck, Uint128(2), Uint128(3)]
             => Ok((vec![RangeCheck, Uint128(5)], 0));
            "uint128_checked_add(2, 3)")]
#[test_case("uint128_checked_sub", vec![], vec![RangeCheck, Uint128(5), Uint128(3)]
             => Ok((vec![RangeCheck, Uint128(2)], 0));
            "uint128_checked_sub(5, 3)")]
#[test_case("uint128_checked_mul", vec![], vec![RangeCheck, Uint128(5), Uint128(3)]
             => Ok((vec![RangeCheck, Uint128(15)], 0));
            "uint128_checked_mul(5, 3)")]
#[test_case("uint128_checked_sub", vec![], vec![RangeCheck, Uint128(3), Uint128(5)]
             => Ok((vec![RangeCheck], 1));
            "uint128_checked_sub(3, 5)")]
fn simulate_branch(
    id: &str,
    generic_args: Vec<GenericArg>,
    inputs: Vec<CoreValue>,
) -> Result<(Vec<CoreValue>, usize), LibFuncSimulationError> {
    simulate(id, generic_args, inputs)
}

/// Tests for simulation of a non branch invocations.
#[test_case("refund_gas", vec![], vec![GasBuiltin(2)] => Ok(vec![GasBuiltin(6)]); "refund_gas(2)")]
#[test_case("array_new", vec![type_arg("uint128")], vec![] => Ok(vec![Array(vec![])]); "array_new()")]
#[test_case("array_append", vec![type_arg("uint128")], vec![Array(vec![]), Uint128(4)] =>
            Ok(vec![Array(vec![Uint128(4)])]); "array_append([], 4)")]
#[test_case("uint128_wrapping_add", vec![], vec![RangeCheck, Uint128(2), Uint128(3)] => Ok(vec![RangeCheck, Uint128(5)]);
            "uint128_wrapping_add(2, 3)")]
#[test_case("uint128_wrapping_sub", vec![], vec![RangeCheck, Uint128(5), Uint128(3)] => Ok(vec![RangeCheck, Uint128(2)]);
            "uint128_wrapping_sub(5, 3)")]
#[test_case("uint128_wrapping_mul", vec![], vec![RangeCheck, Uint128(5), Uint128(3)] => Ok(vec![RangeCheck, Uint128(15)]);
            "uint128_wrapping_mul(5, 3)")]
#[test_case("uint128_div", vec![], vec![RangeCheck, Uint128(32), NonZero(Box::new(Uint128(5)))]
             => Ok(vec![RangeCheck, Uint128(6)]); "uint128_div(32, 5)")]
#[test_case("uint128_mod", vec![], vec![RangeCheck, Uint128(32), NonZero(Box::new(Uint128(5)))]
             => Ok(vec![RangeCheck, Uint128(2)]); "uint128_mod(32, 5)")]
#[test_case("uint128_wrapping_add", vec![value_arg(3)], vec![RangeCheck, Uint128(2)] => Ok(vec![RangeCheck, Uint128(5)]);
            "uint128_wrapping_add<3>(2)")]
#[test_case("uint128_wrapping_sub", vec![value_arg(3)], vec![RangeCheck, Uint128(5)] => Ok(vec![RangeCheck, Uint128(2)]);
            "uint128_wrapping_sub<3>(5)")]
#[test_case("uint128_wrapping_mul", vec![value_arg(3)], vec![RangeCheck, Uint128(5)] => Ok(vec![RangeCheck, Uint128(15)]);
            "uint128_wrapping_mul<3>(5)")]
#[test_case("uint128_div", vec![value_arg(5)], vec![RangeCheck, Uint128(32)] => Ok(vec![RangeCheck, Uint128(6)]);
            "uint128_div<5>(32)")]
#[test_case("uint128_mod", vec![value_arg(5)], vec![RangeCheck, Uint128(32)] => Ok(vec![RangeCheck, Uint128(2)]);
            "uint128_mod<5>(32)")]
#[test_case("uint128_const", vec![value_arg(3)], vec![] => Ok(vec![Uint128(3)]);
            "uint128_const<3>()")]
#[test_case("dup", vec![type_arg("uint128")], vec![Uint128(24)]
             => Ok(vec![Uint128(24), Uint128(24)]); "dup<uint128>(24)")]
#[test_case("drop", vec![type_arg("uint128")], vec![Uint128(2)] => Ok(vec![]); "drop<uint128>(2)")]
#[test_case("unwrap_nz", vec![type_arg("uint128")], vec![NonZero(Box::new(Uint128(6)))]
             => Ok(vec![Uint128(6)]); "unwrap_nz<uint128>(6)")]
#[test_case("store_temp", vec![type_arg("uint128")], vec![Uint128(6)] => Ok(vec![Uint128(6)]);
            "store_temp<uint128>(6)")]
#[test_case("align_temps", vec![type_arg("uint128")], vec![] => Ok(vec![]);
            "align_temps<uint128>()")]
#[test_case("store_local", vec![type_arg("uint128")], vec![Uninitialized, Uint128(6)]
             => Ok(vec![Uint128(6)]); "store_local<uint128>(_, 6)")]
#[test_case("finalize_locals", vec![], vec![] => Ok(vec![]); "finalize_locals()")]
#[test_case("rename", vec![type_arg("uint128")], vec![Uint128(6)] => Ok(vec![Uint128(6)]);
            "rename<uint128>(6)")]
#[test_case("function_call", vec![user_func_arg("drop_all_inputs")], vec![Uint128(3), Uint128(5)]
             => Ok(vec![]); "function_call<drop_all_inputs>()")]
#[test_case("function_call", vec![user_func_arg("identity")], vec![Uint128(3), Uint128(5)]
             => Ok(vec![Uint128(3), Uint128(5)]); "function_call<identity>()")]
fn simulate_none_branch(
    id: &str,
    generic_args: Vec<GenericArg>,
    inputs: Vec<CoreValue>,
) -> Result<Vec<CoreValue>, LibFuncSimulationError> {
    simulate(id, generic_args, inputs).map(|(outputs, chosen_branch)| {
        assert_eq!(chosen_branch, 0);
        outputs
    })
}

#[test_case("get_gas", vec![], vec![RangeCheck, Uninitialized] => MemoryLayoutMismatch;
            "get_gas(empty)")]
#[test_case("get_gas", vec![], vec![] => WrongNumberOfArgs; "get_gas()")]
#[test_case("refund_gas", vec![], vec![Uninitialized] => MemoryLayoutMismatch;
            "refund_gas(empty)")]
#[test_case("refund_gas", vec![], vec![] => WrongNumberOfArgs; "refund_gas()")]
#[test_case("uint128_wrapping_add", vec![], vec![RangeCheck, Uint128(1)] => WrongNumberOfArgs;
            "uint128_wrapping_add(1)")]
#[test_case("uint128_wrapping_sub", vec![], vec![RangeCheck, Uint128(1)] => WrongNumberOfArgs;
            "uint128_wrapping_sub(1)")]
#[test_case("uint128_wrapping_mul", vec![], vec![RangeCheck, Uint128(1)] => WrongNumberOfArgs;
            "uint128_wrapping_mul(1)")]
#[test_case("uint128_div", vec![], vec![RangeCheck, Uint128(1)] => WrongNumberOfArgs; "uint128_div(1)")]
#[test_case("uint128_mod", vec![], vec![RangeCheck, Uint128(1)] => WrongNumberOfArgs; "uint128_mod(1)")]
#[test_case("uint128_wrapping_add", vec![value_arg(3)], vec![RangeCheck] => WrongNumberOfArgs;
            "uint128_wrapping_add<3>()")]
#[test_case("uint128_wrapping_sub", vec![value_arg(3)], vec![RangeCheck] => WrongNumberOfArgs;
            "uint128_wrapping_sub<3>()")]
#[test_case("uint128_wrapping_mul", vec![value_arg(3)], vec![RangeCheck] => WrongNumberOfArgs;
            "uint128_wrapping_mul<3>()")]
#[test_case("uint128_div", vec![value_arg(5)], vec![] => WrongNumberOfArgs; "uint128_div<5>()")]
#[test_case("uint128_mod", vec![value_arg(5)], vec![] => WrongNumberOfArgs; "uint128_mod<5>()")]
#[test_case("uint128_const", vec![value_arg(3)], vec![Uint128(1)] => WrongNumberOfArgs;
            "uint128_const<3>(1)")]
#[test_case("dup", vec![type_arg("uint128")], vec![] => WrongNumberOfArgs; "dup<uint128>()")]
#[test_case("drop", vec![type_arg("uint128")], vec![] => WrongNumberOfArgs; "drop<uint128>()")]
#[test_case("uint128_jump_nz", vec![], vec![] => WrongNumberOfArgs; "uint128_jump_nz()")]
#[test_case("unwrap_nz", vec![type_arg("uint128")], vec![] => WrongNumberOfArgs;
            "unwrap_nz<uint128>()")]
#[test_case("store_temp", vec![type_arg("uint128")], vec![] => WrongNumberOfArgs;
            "store_temp<uint128>()")]
#[test_case("align_temps", vec![type_arg("uint128")], vec![Uint128(1)] => WrongNumberOfArgs;
            "align_temps<uint128>(4)")]
#[test_case("store_local", vec![type_arg("uint128")], vec![] => WrongNumberOfArgs;
            "store_local<uint128>()")]
#[test_case("finalize_locals", vec![], vec![Uint128(4)] => WrongNumberOfArgs; "finalize_locals(4)")]
#[test_case("rename", vec![type_arg("uint128")], vec![] => WrongNumberOfArgs; "rename<uint128>()")]
#[test_case("jump", vec![], vec![Uint128(4)] => WrongNumberOfArgs; "jump(4)")]
#[test_case("function_call", vec![user_func_arg("unimplemented")], vec![] =>
            FunctionSimulationError(
                "unimplemented".into(),
                Box::new(SimulationError::StatementOutOfBounds(StatementIdx(0))));
            "function_call<unimplemented>()")]
fn simulate_error(
    id: &str,
    generic_args: Vec<GenericArg>,
    inputs: Vec<CoreValue>,
) -> LibFuncSimulationError {
    simulate(id, generic_args, inputs).err().unwrap()
}
