/// Type-safe references to Unreal Engine objects
use super::types::{Address, Name};

/// Reference to a property (variable)
#[derive(Debug, Clone, Copy)]
pub struct PropertyRef {
    pub address: Address,
}

impl PropertyRef {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

/// Reference to a function (can be either by address or by name)
#[derive(Debug, Clone)]
pub enum FunctionRef {
    ByAddress(Address),
    ByName(Name),
}

impl FunctionRef {
    pub fn from_address(address: Address) -> Self {
        FunctionRef::ByAddress(address)
    }

    pub fn from_name(name: Name) -> Self {
        FunctionRef::ByName(name)
    }
}

/// Reference to an object
#[derive(Debug, Clone, Copy)]
pub struct ObjectRef {
    pub address: Address,
}

impl ObjectRef {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

/// Reference to a struct type
#[derive(Debug, Clone, Copy)]
pub struct StructRef {
    pub address: Address,
}

impl StructRef {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

/// Reference to a class type
#[derive(Debug, Clone, Copy)]
pub struct ClassRef {
    pub address: Address,
}

impl ClassRef {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}
