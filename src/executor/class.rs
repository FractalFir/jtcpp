use super::{FieldType,UnmetDependency,fatclass::FatClass};
use crate::ExecEnv;
use crate::{IString, Object};
use std::collections::HashMap;
pub(crate) struct Class {
    class_id: usize,
    virtual_methods: Box<[usize]>,
    statics: Box<[usize]>,
    statics_map: HashMap<IString, usize>,
    field_map: HashMap<IString, usize>,
    field_types: Box<[FieldType]>,
}
impl Class {
    pub(crate) fn get_field(&self,name:&str)->Option<(usize,&FieldType)>{
        println!("{:?}",self.field_map);
        let field_id = *self.field_map.get(name)?;
        let ftype = &self.field_types[field_id];
        Some((field_id,ftype))
    }
    pub(crate) fn empty() -> Self {
        Self {
            class_id: 0,
            virtual_methods: Box::new([]),
            statics: Box::new([]),
            statics_map: HashMap::new(),
            field_map: HashMap::new(),
            field_types: Box::new([]),
        }
    }
    pub(crate) fn new(&self) -> Object {
        let class_id: usize = self.class_id;
        //Value::Void;values: Box<[Value]>
        //
        let mut values = Vec::with_capacity(self.field_types.len());
        for field_type in self.field_types.iter() {
            values.push(field_type.default_value());
        }
        Object::Object {
            class_id,
            values: values.into(),
        }
    }
}
pub(crate) fn finalize(class: &FatClass, env: &mut ExecEnv) -> Result<Class, UnmetDependency> {
    let super_name = class.super_name();
    let super_class = env.code_container.lookup_class(super_name);
    let super_class = if let Some(super_class) = super_class {
        super_class
    } else {
        return Err(UnmetDependency::NeedsClass(super_name.into()));
    };
    let mut statics_map:HashMap<IString, usize> = HashMap::with_capacity(class.static_fields().len());
    let mut statics = Vec::with_capacity(class.fields().len());
    for (static_name,ftype) in class.static_fields(){
        let static_id = env.env_mem.insert_static(ftype.default_value());
        statics.push(static_id);
        statics_map.insert(static_name.clone(),static_id);
    }
    let mut field_types = Vec::with_capacity(class.fields().len());
    let mut field_map:HashMap<IString, usize> = HashMap::with_capacity(class.fields().len());
    for (field_name,ftype) in class.fields(){
        field_map.insert(field_name.clone(),field_types.len());
        field_types.push((*ftype).clone());
    }
    let class_id = 0;
    Ok(Class {
        class_id,
        virtual_methods: Box::new([]),
        statics: statics.into(),
        statics_map,
        field_map,
        field_types: field_types.into(),
    })
}
