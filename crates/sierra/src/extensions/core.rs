use super::ap_tracking::RevokeApTrackingLibFunc;
use super::array::{ArrayLibFunc, ArrayType};
use super::dict_felt_to::{DictFeltToLibFunc, DictFeltToType};
use super::drop::DropLibFunc;
use super::duplicate::DupLibFunc;
use super::enm::{EnumLibFunc, EnumType};
use super::modules::boxing::{BoxLibFunc, BoxType};
use super::modules::felt::{FeltLibFunc, FeltType};
use super::modules::function_call::FunctionCallLibFunc;
use super::modules::gas::{GasBuiltinType, GasLibFunc};
use super::modules::integer::{Uint128LibFunc, Uint128Type};
use super::modules::mem::MemLibFunc;
use super::modules::non_zero::{NonZeroType, UnwrapNonZeroLibFunc};
use super::modules::syscalls::SyscallPtrType;
use super::modules::unconditional_jump::UnconditionalJumpLibFunc;
use super::pedersen::{PedersenLibFunc, PedersenType};
use super::range_check::RangeCheckType;
use super::squashed_dict_felt_to::SquashedDictFeltToType;
use super::strct::{StructLibFunc, StructType};
use super::uninitialized::UninitializedType;
use crate::{define_libfunc_hierarchy, define_type_hierarchy};

define_type_hierarchy! {
    pub enum CoreType {
        Array(ArrayType),
        Box(BoxType),
        Felt(FeltType),
        GasBuiltin(GasBuiltinType),
        Uint128(Uint128Type),
        NonZero(NonZeroType),
        RangeCheck(RangeCheckType),
        Uninitialized(UninitializedType),
        Enum(EnumType),
        Struct(StructType),
        DictFeltTo(DictFeltToType),
        SquashedDictFeltTo(SquashedDictFeltToType),
        Pedersen(PedersenType),
        SyscallPtr(SyscallPtrType),
    }, CoreTypeConcrete
}

define_libfunc_hierarchy! {
    pub enum CoreLibFunc {
        ApTracking(RevokeApTrackingLibFunc),
        Array(ArrayLibFunc),
        Box(BoxLibFunc),
        Drop(DropLibFunc),
        Dup(DupLibFunc),
        Felt(FeltLibFunc),
        FunctionCall(FunctionCallLibFunc),
        Gas(GasLibFunc),
        Uint128(Uint128LibFunc),
        Mem(MemLibFunc),
        UnwrapNonZero(UnwrapNonZeroLibFunc),
        UnconditionalJump(UnconditionalJumpLibFunc),
        Enum(EnumLibFunc),
        Struct(StructLibFunc),
        DictFeltTo(DictFeltToLibFunc),
        Pedersen(PedersenLibFunc),
    }, CoreConcreteLibFunc
}
