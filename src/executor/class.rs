use super::{fatclass::FatClass, FieldType, UnmetDependency};
use crate::ClassRef;
use crate::ExecEnv;
use crate::{IString, Object};
use std::collections::HashMap;
pub(crate) struct Class {
    class_id: ClassRef,
    class_name: IString,
    virtual_methods: Box<[usize]>,
    virtual_map: HashMap<IString, usize>,
    //statics: Box<[usize]>,
    statics_map: HashMap<IString, usize>,
    field_map: HashMap<IString, usize>,
    field_types: Box<[FieldType]>,
}
impl Class {
    pub(crate) fn statics_map(&self) -> &HashMap<IString, usize> {
        &self.statics_map
    }
    pub(crate) fn virtual_methods(&self) -> &[usize] {
        &self.virtual_methods
    }
    pub(crate) fn virtual_map(&self) -> &HashMap<IString, usize> {
        &self.virtual_map
    }
    pub(crate) fn field_types(&self) -> &[FieldType] {
        &self.field_types
    }
    pub(crate) fn field_map(&self) -> &HashMap<IString, usize> {
        &self.field_map
    }
    pub(crate) fn class_name(&self) -> &str {
        &self.class_name
    }
    //Call *ONLY* once per class, when after adding it to CodeContainer!
    pub(crate) fn set_id(&mut self, id: ClassRef) {
        assert_eq!(self.class_id, 0, "Tried to set class id after linknig!");
        self.class_id = id;
    }
    pub(crate) fn get_virtual(&self, virtual_id: usize) -> Option<usize> {
        self.virtual_methods.get(virtual_id).copied()
    }
    pub(crate) fn lookup_virtual(&self, method_name: &str) -> Option<usize> {
        if let Some(idx) = self.virtual_map.get(method_name.into()) {
            Some(*idx)
        } else {
            panic!(
                "Could not find method:{method_name} in class {} that has methods:{:?}",
                self.class_name, self.virtual_map
            );
        }
    }
    pub(crate) fn get_field(&self, name: &str) -> Option<(usize, &FieldType)> {
        //println!("{:?}",self.field_map);
        let field_id = *self.field_map.get(name)?;
        let ftype = &self.field_types[field_id];
        Some((field_id, ftype))
    }
    pub(crate) fn get_static(&self, name: &str) -> Option<usize> {
        //println!("{:?}",self.field_map);
        let field_id = *self.statics_map.get(name)?;
        Some(field_id)
    }
    pub(crate) fn empty() -> Self {
        Self {
            class_name: "".into(),
            class_id: 0,
            virtual_methods: Box::new([]),
            virtual_map: HashMap::new(),
            //statics: Box::new([]),
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
    ///TODO: Propely handle inheretence!
    let super_name = class.super_name();
    let super_class = env.code_container.lookup_class(super_name);
    let super_class = if let Some(super_class) = super_class {
        super_class
    } else {
        return Err(UnmetDependency::NeedsClass(super_name.into()));
    };
    let mut statics_map: HashMap<IString, usize> = env.get_class_statics_map(super_class).unwrap().clone();
    //let mut statics = Vec::with_capacity(class.fields().len());
    for (static_name, ftype) in class.static_fields() {
        let static_id = env.env_mem.insert_static(ftype.default_value());
        //statics.push(static_id);
        statics_map.insert(static_name.clone(), static_id);
    }
    let mut field_types: Vec<FieldType> = env.get_class_field_types(super_class).unwrap().into(); //Vec::with_capacity(class.fields().len());
    let mut field_map: HashMap<IString, usize> = env.get_class_field_map(super_class).unwrap();
    ///HashMap::with_capacity(class.fields().len());
    for (field_name, ftype) in class.fields() {
        field_map.insert(field_name.clone(), field_types.len());
        field_types.push((*ftype).clone());
    }
    //This is naive and *DOES NOT* respect inheretnce!
    let mut virtual_methods: Vec<usize> =
        env.get_class_virtual_methods(super_class).unwrap().into();
    let mut virtual_map: HashMap<IString, usize> =
        env.get_class_virtual_map(super_class).unwrap().into(); //HashMap::new();
    for (virtual_method, real_method) in class.virtuals() {
        //println!("virtual_method:{virtual_method},real_method:{real_method}");
        if let Some(id) = virtual_map.get(virtual_method) {
            if virtual_methods.len() <= *id {
                println!("VMethod with invalid vid! Method comes from super class {super_name}, and is named {virtual_method} it's vi is: {id}.
                    Inserting 'InvalidMethod' to try and go forward anyway.");
            }
            else{
                virtual_methods[*id] = env.code_container.lookup_or_insert_method(real_method);
            }  
        } else {
            virtual_methods.push(env.code_container.lookup_or_insert_method(real_method));
            virtual_map.insert(virtual_method.to_owned(), virtual_methods.len() - 1);
        }
    }
    
    let class_id = 0;
    //TODO: Save info about real methods implementing an inteface.
    for interface_name in class.interfaces(){
        let interface_class = env.code_container.lookup_class(interface_name);
        let interface_class = if let Some(interface_class) = interface_class {
        interface_class
        } else {
            return Err(UnmetDependency::NeedsClass(interface_name.clone()));
        };
        let interface_vmap: HashMap<IString, usize> = env.get_class_virtual_map(interface_class).unwrap().into();
        let interface_vmethods: Vec<usize> = env.get_class_virtual_methods(interface_class).unwrap().into(); 
        for (name,vid) in interface_vmap.iter(){
            virtual_map.entry(name.clone()).or_insert_with(||{
                if let Some(real_method) = interface_vmethods.get(*vid){
                    *real_method
                }
                else{
                    println!("VMethod with invalid vid! Method comes from interface {interface_name}, and is named {name} it's vi is: {vid}.
                    Inserting 'InvalidMethod' to try and go forward anyway.");
                    env.code_container.lookup_or_insert_method("InvalidMethod")
                }
                //panic!("{name} seems to be a deafult interface method! It's real method likely is {real_method}!");
            });
        }
        //panic!("Can't yet handle interfaces yet! interface_vmap:{interface_vmap:?} interface_vmethods:{interface_vmethods:?}");
    }
    Ok(Class {
        class_name: class.class_name().into(),
        class_id,
        virtual_methods: virtual_methods.into(),
        virtual_map,
        //statics: statics.into(),
        statics_map,
        field_map,
        field_types: field_types.into(),
    })
}
