use crate::{IString,Class,Method,ObjectRef,ClassRef,MethodRef,StaticRef,FieldRef,FatClass,Object};
use crate::executor::FieldType;
use std::collections::HashMap;
pub(crate) struct UnfinalizedMethod {
    pub(crate) ops: Box<[crate::executor::fatops::FatOp]>,
    pub(crate) method_id: MethodRef,
}
pub(crate) struct CodeContainer {
    classes: Vec<Class>,
    class_names: HashMap<IString, usize>,
    methods: Vec<Option<Method>>,
    method_names: HashMap<IString, usize>,
    static_strings: HashMap<IString, ObjectRef>,
    const_classes: HashMap<ClassRef, ObjectRef>,
    // have some unresolved dependencies.
    pub(crate) unfinalized_classes: HashMap<IString, Vec<FatClass>>,
    pub(crate) unfinalized_methods: HashMap<IString, Vec<UnfinalizedMethod>>,
}
impl CodeContainer {
    pub(crate) fn class_alias(&mut self,class_name:&str,prefix:&str){
        println!("Inserting class alias to {class_name} -> {prefix}{class_name}");
        match self.class_names.get(class_name){
            Some(id)=>{self.class_names.insert(format!("{prefix}{class_name}").into(),*id);},
            None=>(),
        }
    }
    pub(crate) fn const_classes(&mut self)->&mut HashMap<ClassRef, ObjectRef>{
        &mut self.const_classes
    }
    pub(crate) fn static_strings(&mut self)->&mut HashMap<IString, ObjectRef>{
        &mut self.static_strings
    }
    pub(crate) fn get_class(&self,class_id:ClassRef)->Option<&Class>{
        self.classes.get(class_id)
    }
    pub(crate) fn method_names(&self) -> &HashMap<IString, usize>{
        &self.method_names
    }
    pub(crate) fn new_obj(&self,class_id:ClassRef)->Object{
        self.classes[class_id].new()
    }
    pub(crate) fn lookup_virtual(&self,class_id:ClassRef,method_name:&str)->Option<StaticRef>{
        self.classes.get(class_id)?.lookup_virtual(method_name)
    }
    pub(crate) fn get_static(&self,class_id:ClassRef,static_name:&str)->Option<StaticRef>{
        self.classes.get(class_id)?.get_static(static_name) 
    }
    pub(crate) fn get_field(&self,class_id:ClassRef,field_name:&str)->Option<(FieldRef, &FieldType)>{
        self.classes.get(class_id)?.get_field(field_name)
    }
    pub(crate) fn get_virtual(&self, class: ClassRef, id: usize) -> Option<usize> {
        self.classes.get(class)?.get_virtual(id)
    }
    pub(crate) fn unfinalized_class_count(&self) -> usize{
        self.unfinalized_classes.len()
    }
    //pub fn lookup_virutal(&self,id:
    pub(crate) fn lookup_class(&self, name: &str) -> Option<usize> {
        //println!("class_names:{:?}",self.class_names);
        self.class_names.get(name).copied()
    }
    pub(crate) fn methods(&self)->&Vec<Option<Method>>{
        &self.methods
    }
    pub(crate) fn methods_mut(&mut self)->&mut Vec<Option<Method>>{
        &mut self.methods
    }
    pub(crate) fn add_unfinalized_class(&mut self, dependency:&str, base_class:FatClass){
         self.unfinalized_classes.entry(dependency.into())
                    .or_insert(Vec::new())
                    .push(base_class);
    }
    pub(crate) fn classes(&self)->&[Class]{
         &self.classes
    }
    pub(crate) fn set_or_replace_class(&mut self, name: &str, mut class: Class) -> usize {
        let idx = *self
            .class_names
            .entry(name.to_owned().into_boxed_str())
            .or_insert_with(|| {
                self.classes.push(Class::empty());
                self.classes.len() - 1
            });
        class.set_id(idx);
        self.classes[idx] = class;
        idx
    }
    pub(crate) fn lookup_or_insert_method(&mut self, name: &str) -> usize {
        *self
            .method_names
            .entry(name.to_owned().into_boxed_str())
            .or_insert_with(|| {
                self.methods.push(None);
                self.methods.len() - 1
            })
    }
    fn method_name(&self,id:usize)->Option<&str>{
        for (name,method_id) in &self.method_names{
            if id == *method_id{
                return Some(name);
            }
        }
        None
    }
    //  unfinalized_classes: HashMap<IString, Vec<FatClass>>,
    fn class_dependency(&self,name:&str)->Option<&str>{
        // unfinalized_methods: HashMap<IString, Vec<UnfinalizedMethod>>,
        println!("class name:\"{name}\"");
        if let Some(id) = self.lookup_class(name){
            println!("Class {name} exists, and is finalised, with id {id}!");
        }
        for (reason,classes) in &self.unfinalized_classes{
            for class in classes{
                if class.class_name() == name{
                    return Some(reason);
                }
            }
        }
        None
    }
    fn method_dependency(&self,id:usize)->Option<&str>{
        // unfinalized_methods: HashMap<IString, Vec<UnfinalizedMethod>>,
        for (reason,methods) in &self.unfinalized_methods{
            for method in methods{
                if method.method_id == id{
                    return Some(reason);
                }
            }
        }
        None
    }
    pub(crate) fn diagnose_method(&self,method_id:usize){
            let name = self.method_name(method_id).unwrap();
            let dep = self.method_dependency(method_id).unwrap();
            println!("method with name {name} and ID {method_id} is missing, because it depends on {dep}!");
            let mut dep = dep;
            //let mut limiter = 0;
            while let Some(class_dep) = self.class_dependency(dep){
                println!("{dep} depends on {class_dep}!");
                if self.lookup_class(class_dep).is_some(){
                    println!("Which now exists, but {dep} does not know that.");
                    return;
                }
                else{
                    dep = class_dep;
                }
                //limiter += 1;
                /*
                if limiter > 100{
                    panic!("Loopty loop!");
                }*/
            }
    }
    pub(crate) fn get_method(&self,method_id:MethodRef)->Option<&Option<Method>>{
        self.methods.get(method_id)
    }
    pub(crate) fn new() -> Self {
        let object_class = Class::empty();
        let methods = Vec::new();
        let classes = vec![];
        let class_names = HashMap::with_capacity(0x100);
        let method_names = HashMap::with_capacity(0x100);
        let mut res = Self {
            methods,
            classes,
            class_names,
            method_names,
            unfinalized_classes: HashMap::new(),
            unfinalized_methods: HashMap::new(),
            static_strings: HashMap::new(),
            const_classes: HashMap::new(),
        };
        res.set_or_replace_class("java/lang/Object", object_class);
        res
    }
    //fn set_meth
}
