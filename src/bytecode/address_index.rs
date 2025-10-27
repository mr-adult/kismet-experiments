use std::collections::BTreeMap;

use jmap::{ObjectType, Property};

use super::types::Address;

#[derive(Debug, Clone)]
pub struct ObjectInfo<'a> {
    pub path: &'a str,
    pub object: &'a ObjectType,
}

#[derive(Debug, Clone)]
pub struct PropertyInfo<'a> {
    pub owner: ObjectInfo<'a>,
    pub property: &'a Property,
}

pub struct AddressIndex<'a> {
    pub jmap: &'a jmap::Jmap,
    pub object_index: BTreeMap<u64, &'a str>, // address => object path
    pub property_index: BTreeMap<u64, (&'a str, usize)>, // address => (owner path, property index)
}

impl<'a> AddressIndex<'a> {
    pub fn new(jmap: &'a jmap::Jmap) -> Self {
        let mut object_index = BTreeMap::new();
        let mut property_index = BTreeMap::new();

        // Index objects by address
        for (path, obj) in &jmap.objects {
            let address = obj.get_object().address.0;
            object_index.insert(address, path.as_str());
        }

        // Index properties by address
        for (path, obj) in &jmap.objects {
            if let Some(struct_obj) = obj.get_struct() {
                for (prop_idx, prop) in struct_obj.properties.iter().enumerate() {
                    property_index.insert(prop.address.0, (path.as_str(), prop_idx));
                }
            }
        }

        Self {
            jmap,
            object_index,
            property_index,
        }
    }

    pub fn resolve_object(&self, address: Address) -> Option<ObjectInfo<'_>> {
        self.object_index
            .get(&address.as_u64())
            .map(|path| ObjectInfo {
                object: self.jmap.objects.get(*path).unwrap(),
                path,
            })
    }

    pub fn resolve_property(&self, address: Address) -> Option<PropertyInfo<'_>> {
        self.property_index
            .get(&address.as_u64())
            .map(|(path, prop_idx)| {
                let object = self.jmap.objects.get(*path).unwrap();
                PropertyInfo {
                    property: &object.get_struct().unwrap().properties[*prop_idx],
                    owner: ObjectInfo { object, path },
                }
            })
    }
}
