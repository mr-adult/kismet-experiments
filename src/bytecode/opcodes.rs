/// Unreal Engine Kismet bytecode opcodes

macro_rules! define_opcodes {
    ($(($value:expr, $variant:ident)),* $(,)?) => {
        // EExprToken enum - all bytecode opcodes
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum EExprToken {
            $($variant,)*
            Unknown(u8),
        }

        impl EExprToken {
            pub fn opcode_value(&self) -> u8 {
                match self {
                    $(EExprToken::$variant => $value,)*
                    EExprToken::Unknown(val) => *val,
                }
            }
        }

        impl From<u8> for EExprToken {
            fn from(value: u8) -> Self {
                match value {
                    $($value => EExprToken::$variant,)*
                    _ => EExprToken::Unknown(value),
                }
            }
        }
    };
}

define_opcodes! {
    (0x00, LocalVariable),
    (0x01, InstanceVariable),
    (0x02, DefaultVariable),
    (0x04, Return),
    (0x06, Jump),
    (0x07, JumpIfNot),
    (0x09, Assert),
    (0x0B, Nothing),
    (0x0C, NothingInt32),
    (0x0F, Let),
    (0x11, BitFieldConst),
    (0x12, ClassContext),
    (0x13, MetaCast),
    (0x14, LetBool),
    (0x15, EndParmValue),
    (0x16, EndFunctionParms),
    (0x17, Self_),
    (0x18, Skip),
    (0x19, Context),
    (0x1A, ContextFailSilent),
    (0x1B, VirtualFunction),
    (0x1C, FinalFunction),
    (0x1D, IntConst),
    (0x1E, FloatConst),
    (0x1F, StringConst),
    (0x20, ObjectConst),
    (0x21, NameConst),
    (0x22, RotationConst),
    (0x23, VectorConst),
    (0x24, ByteConst),
    (0x25, IntZero),
    (0x26, IntOne),
    (0x27, True),
    (0x28, False),
    (0x29, TextConst),
    (0x2A, NoObject),
    (0x2B, TransformConst),
    (0x2C, IntConstByte),
    (0x2D, NoInterface),
    (0x2E, DynamicCast),
    (0x2F, StructConst),
    (0x30, EndStructConst),
    (0x31, SetArray),
    (0x32, EndArray),
    (0x33, PropertyConst),
    (0x34, UnicodeStringConst),
    (0x35, Int64Const),
    (0x36, UInt64Const),
    (0x38, PrimitiveCast),
    (0x39, SetSet),
    (0x3A, EndSet),
    (0x3B, SetMap),
    (0x3C, EndMap),
    (0x3D, SetConst),
    (0x3E, EndSetConst),
    (0x3F, MapConst),
    (0x40, EndMapConst),
    (0x42, StructMemberContext),
    (0x43, LetMulticastDelegate),
    (0x44, LetDelegate),
    (0x45, LocalVirtualFunction),
    (0x46, LocalFinalFunction),
    (0x48, LocalOutVariable),
    (0x4A, DeprecatedOp4A),
    (0x4B, InstanceDelegate),
    (0x4C, PushExecutionFlow),
    (0x4D, PopExecutionFlow),
    (0x4E, ComputedJump),
    (0x4F, PopExecutionFlowIfNot),
    (0x50, Breakpoint),
    (0x51, InterfaceContext),
    (0x52, ObjToInterfaceCast),
    (0x53, EndOfScript),
    (0x54, CrossInterfaceCast),
    (0x55, InterfaceToObjCast),
    (0x5A, WireTracepoint),
    (0x5B, SkipOffsetConst),
    (0x5C, AddMulticastDelegate),
    (0x5D, ClearMulticastDelegate),
    (0x5E, Tracepoint),
    (0x5F, LetObj),
    (0x60, LetWeakObjPtr),
    (0x61, BindDelegate),
    (0x62, RemoveMulticastDelegate),
    (0x63, CallMulticastDelegate),
    (0x64, LetValueOnPersistentFrame),
    (0x65, ArrayConst),
    (0x66, EndArrayConst),
    (0x67, SoftObjectConst),
    (0x68, CallMath),
    (0x69, SwitchValue),
    (0x6A, InstrumentationEvent),
    (0x6B, ArrayGetByRef),
    (0x6C, ClassSparseDataVariable),
    (0x6D, FieldPathConst),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EBlueprintTextLiteralType {
    Empty = 0,
    LocalizedText = 1,
    InvariantText = 2,
    LiteralString = 3,
    StringTableEntry = 4,
}

impl From<u8> for EBlueprintTextLiteralType {
    fn from(value: u8) -> Self {
        match value {
            0 => EBlueprintTextLiteralType::Empty,
            1 => EBlueprintTextLiteralType::LocalizedText,
            2 => EBlueprintTextLiteralType::InvariantText,
            3 => EBlueprintTextLiteralType::LiteralString,
            4 => EBlueprintTextLiteralType::StringTableEntry,
            _ => EBlueprintTextLiteralType::Empty,
        }
    }
}
