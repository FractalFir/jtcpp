pub(crate) type IString = Box<str>;
use crate::importer::opcodes::OpCode;
use std::collections::HashMap;
mod executor;
mod importer;
pub type ObjectRef = usize;
pub type ClassRef = usize;
use crate::executor::baseir::BaseIR;
use executor::class::Class;
use executor::ExecCtx;
enum Method {
    BaseIR { ops: Box<[BaseIR]> },
    Invokable(Box<dyn Invokable>),
}
trait Invokable {
    fn call(
        &self,
        args: &[Value],
        memory: &mut EnvMemory,
        code_container: &CodeContainer,
    ) -> Result<Value, ExecException>;
}
impl Method {
    fn call(
        &self,
        ctx:ExecCtx
    ) -> Result<Value, ExecException> {
        match self {
            //Self::Invokable(invokable) => invokable.call(args, memory, code_container),
            Method::BaseIR { ops } => executor::baseir::call(ops, ctx),
            _=>todo!("Can't call externs yet!")
        }
    }
}
#[derive(Clone,Copy, Debug, PartialEq)]
enum Value {
    Void,
    Int(i32),
    ObjectRef(ObjectRef),
    Float(f32),
}
impl Value {
    fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(a) => Some(*a),
            _ => None,
        }
    }
    fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(a) => Some(*a),
            _ => None,
        }
    }
    fn as_objref(&self) -> Option<ObjectRef> {
        match self {
            Value:: ObjectRef(id) => Some(*id),
            _ => None,
        }
    }
}
enum Object {
    Object {
        class_id: usize,
        values: Box<[Value]>,
    },
    String(IString),
}
impl Object{
    pub fn set_field(&mut self,id:usize,value:Value){
        //println!("seting field {id} to {value:?}");
        match self{
             Self::Object {values,..}=>values[id] = value,
             _=>(),
        }
    }
    pub fn get_field(&self,id:usize)->Option<Value>{
        match self{
             Self::Object {values,..}=>values.get(id).cloned(),
             _=>None,
        }
    }
}
struct EnvMemory {
    objects: Vec<Object>,
    statics: Vec<Value>,
    lock:std::sync::Mutex<()>,
}
impl EnvMemory {
    fn new_obj(this:*mut Self,new_obj:Object)->ObjectRef{
        unsafe{(*this).objects.push(new_obj)};
        unsafe{(*this).objects.len() - 1}
    }
    fn get_static(this:*const Self,index:usize)->Value{
        let lock = unsafe{(*this).lock.lock().expect("poisoned mutex!")};
        let val = unsafe{(*this).statics[index]};
        drop(lock);
        val
    }
    pub(crate) fn get_field(this:*const Self,obj_ref:ObjectRef,field_id:usize)->Option<Value>{
        let lock = unsafe{(*this).lock.lock().expect("poisoned mutex!")};
        let val = unsafe{(*this).objects[obj_ref].get_field(field_id)};
        drop(lock);
        val
    }
    fn set_field(this:*mut Self,obj_ref:ObjectRef,field_id:usize,value:Value){
        let lock = unsafe{(*this).lock.lock().expect("poisoned mutex!")};
        unsafe{(*this).objects[obj_ref].set_field(field_id,value)};
        drop(lock);
    }
    fn set_static(this:*mut Self,index:usize,value:Value){
        let lock = unsafe{(*this).lock.lock().expect("poisoned mutex!")};
        unsafe{(*this).statics[index] = value};
        drop(lock);
    }
    pub(crate) fn insert_static(&mut self,value:Value)->usize{
        let lock = self.lock.lock().expect("poisoned mutex!");
        let index = self.statics.len();
        self.statics.push(value);
        drop(lock);
        index
    }
    fn new() -> Self {
        Self {
            objects: Vec::with_capacity(0x100),
            statics: Vec::with_capacity(0x100),
            lock:std::sync::Mutex::new(()),
        }
    }
}
struct CodeContainer {
    classes: Vec<Class>,
    class_names: HashMap<IString, usize>,
    methods: Vec<Option<Method>>,
    method_names: HashMap<IString, usize>,
}
impl CodeContainer {
    fn lookup_class(&self, name: &str) -> Option<usize> {
        //println!("class_names:{:?}",self.class_names);
        self.class_names.get(name).copied()
    }
    fn set_or_replace_class(&mut self, name: &str, class: Class) -> usize {
        let idx = *self
            .class_names
            .entry(name.to_owned().into_boxed_str())
            .or_insert_with(|| {
                self.classes.push(Class::empty());
                self.classes.len() - 1
            });
        self.classes[idx] = class;
        idx
    }
    fn lookup_or_insert_method(&mut self, name: &str) -> usize {
        *self
            .method_names
            .entry(name.to_owned().into_boxed_str())
            .or_insert_with(|| {
                self.methods.push(None);
                self.methods.len() - 1
            })
    }
    fn new() -> Self {
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
        };
        res.set_or_replace_class("java/lang/Object", object_class);
        res
    }
    //fn set_meth
}
struct ExecEnv {
    code_container: CodeContainer,
    env_mem: EnvMemory,
    //objects:Vec<Option<Object>>
}
#[test]
fn arg_counter() {
    assert_eq!(method_desc_to_argc("()I"), 0);
    assert_eq!(method_desc_to_argc("(I)I"), 1);
    assert_eq!(method_desc_to_argc("(IL)I"), 2);
    assert_eq!(method_desc_to_argc("(ILF)I"), 3);
    assert_eq!(method_desc_to_argc("(ILF)"), 3);
}
fn method_desc_to_argc(desc: &str) -> u8 {
    assert_eq!(desc.chars().nth(0), Some('('));
    let mut char_beg = 0;
    let mut char_end = 0;
    for (index, character) in desc.chars().enumerate() {
        if character == '(' {
            assert_eq!(char_beg, 0);
            char_beg = index;
        } else if character == ')' {
            assert_eq!(char_end, 0);
            char_end = index;
        }
    }
    (char_end - char_beg - 1) as u8
}
fn mangle_method_name(class: &str, method: &str, desc: &str) -> IString {
    format!("{class}::{method}{desc}").into_boxed_str()
}
impl ExecEnv {
    fn lookup_or_insert_static(&mut self, class: usize, static_field: &str) -> usize {
        todo!("Can't insert statics yet!");
    }
    pub fn new() -> Self {
        let env_mem = EnvMemory::new();
        let code_container = CodeContainer::new();
        //let objects = vec!
        Self {
            code_container,
            env_mem,
        }
    }
    pub(crate) fn load_method(
        &mut self,
        method: &crate::importer::Method,
        class: &crate::importer::ImportedJavaClass,
    ) {
        let bytecode = if let Some(bytecode) = method.bytecode() {
            bytecode
        } else {
            self.code_container.methods.push(None);
            return;
        };
        let fat = crate::executor::fatops::expand_ops(bytecode, &class);
        println!("fat:{fat:?}");
        let baseir = crate::executor::baseir::into_base(&fat, self).unwrap();
        let max_locals = method.max_locals().unwrap();
        let (name, descriptor) = (method.name_index(), method.descriptor_index());
        let method_class = class.lookup_class(class.this_class()).unwrap();
        let name = class.lookup_utf8(name).unwrap();
        let descriptor = class.lookup_utf8(descriptor).unwrap();
        let mangled = mangle_method_name(method_class, name, descriptor);
        let method_id = self.code_container.lookup_or_insert_method(&mangled);
        let argc = method_desc_to_argc(&descriptor);
        let af = method.access_flags();
        let is_static = af.is_static();
        //println!("mangled:{mangled}");

        self.code_container.methods[method_id] = Some(Method::BaseIR { ops: baseir });
    }
    pub(crate) fn load_class(&mut self, class: crate::importer::ImportedJavaClass) {
        let base_class = crate::executor::fatclass::expand_class(&class);
        let final_class = crate::executor::class::finalize(&base_class, self).unwrap();
        self.code_container
            .set_or_replace_class(base_class.class_name(), final_class);
        for method in class.methods() {
            self.load_method(method, &class);
        }
    }
    pub(crate) fn lookup_method(&self, method_name: &str) -> Option<usize> {
        self.code_container.method_names.get(method_name).copied()
    }
    pub(crate) fn lookup_class(&self, class_name: &str) -> Option<usize> {
        self.code_container.lookup_class(class_name)
    }
    pub(crate) fn new_obj(&mut self,class:ClassRef) -> ObjectRef{
        let new_obj = self.code_container.classes[class].new();
        EnvMemory::new_obj(&mut self.env_mem as *mut _,new_obj)
    }
    pub(crate) fn call_method(
        &mut self,
        method_id: usize,
        args: &[Value],
    ) -> Result<Value, ExecException> {
        let mut args: Vec<_> = args.into();
                //args.reverse();
        let e_ctx = executor::ExecCtx::new(&mut self.env_mem,&self.code_container,&args,6);
        //todo!();
        let method = self.code_container.methods.get(method_id);
        method
            .ok_or(ExecException::MethodNotFound)?
            .as_ref()
            .ok_or(ExecException::MethodNotFound)?
            .call(e_ctx)
    }
}
#[derive(Debug)]
enum ExecException {
    MethodNotFound,
}
#[test]
fn exec_identity() {
    let mut file = std::fs::File::open("test/Identity.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let identity = exec_env
        .lookup_method(&mangle_method_name("Identity", "Identity", "(I)I"))
        .unwrap();
    for a in 0..1000 {
        assert_eq!(
            exec_env.call_method(identity, &[Value::Int(a)]).unwrap(),
            Value::Int(a)
        );
    }
}

#[test]
fn basic_arthm() {
    let mut file = std::fs::File::open("test/BasicArthm.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let add = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Add", "(II)I"))
        .unwrap();
    let sub = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Sub", "(II)I"))
        .unwrap();
    let mul = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Mul", "(II)I"))
        .unwrap();
    let div = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Div", "(II)I"))
        .unwrap();
    let rem = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Mod", "(II)I"))
        .unwrap();
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env
                    .call_method(add, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a + b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env
                    .call_method(sub, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a - b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env
                    .call_method(mul, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a * b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env
                    .call_method(div, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a / b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env
                    .call_method(rem, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a % b)
            );
        }
    }
}
struct AddFiveInvokable;
struct SqrtInvokable;
impl Invokable for AddFiveInvokable {
    fn call(
        &self,
        args: &[Value],
        memory: &mut EnvMemory,
        code_container: &CodeContainer,
    ) -> Result<Value, ExecException> {
        Ok(Value::Int(args[0].as_int().unwrap() + 5))
    }
}
impl Invokable for SqrtInvokable {
    fn call(
        &self,
        args: &[Value],
        memory: &mut EnvMemory,
        code_container: &CodeContainer,
    ) -> Result<Value, ExecException> {
        Ok(Value::Float(args[0].as_float().unwrap().sqrt()))
    }
}
#[test]
fn exec_call() {
    let mut file = std::fs::File::open("test/Calls.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let sqr_mag = exec_env
        .lookup_method(&mangle_method_name("Calls", "SqrMag", "(III)I"))
        .unwrap();
    let first = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnFirst", "(IIIII)I"))
        .unwrap();
    let second = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnSecond", "(IIIII)I"))
        .unwrap();
    let last = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnLast", "(IIIII)I"))
        .unwrap();
    let first_bck = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnFirst", "(IIIII)I"))
        .unwrap();
    assert_eq!(
        exec_env
            .call_method(
                first_bck,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(1)
    );
    assert_eq!(
        exec_env
            .call_method(
                first,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(1)
    );
    assert_eq!(
        exec_env
            .call_method(
                second,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(2)
    );
    assert_eq!(
        exec_env
            .call_method(
                last,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(5)
    );
    for a in 0..1000 {
        exec_env
            .call_method(sqr_mag, &[Value::Int(a), Value::Int(7), Value::Int(8)])
            .unwrap();
    }
    let extern_call = exec_env
        .lookup_method(&mangle_method_name("Calls", "ExternCallTest", "(I)I"))
        .unwrap();
    for a in -1000..1000 {
        assert_eq!(
            exec_env.call_method(extern_call, &[Value::Int(a)]).unwrap(),
            Value::Int(0)
        );
    }
    exec_env.code_container.methods[extern_call] =
        Some(Method::Invokable(Box::new(AddFiveInvokable)));
    for a in -1000..1000 {
        assert_eq!(
            exec_env.call_method(extern_call, &[Value::Int(a)]).unwrap(),
            Value::Int(a + 5)
        );
    }
}

#[test]
fn exec_hw() {
    let mut file = std::fs::File::open("test/HelloWorld.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);

    let hw = exec_env
        .lookup_method(&mangle_method_name(
            "HelloWorld",
            "main",
            "([Ljava/lang/String;)V",
        ))
        .unwrap();
    exec_env.call_method(hw, &[]).unwrap();
}
#[test]
fn fields() {
    let mut file = std::fs::File::open("test/Fields.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    
    //let hw = exec_env.lookup_method(&mangle_method_name("HelloWorld","main","([Ljava/lang/String;)V")).unwrap();
    //exec_env.call_method(hw,&[]).unwrap();
}
#[test]
fn gravity() {
    let mut file = std::fs::File::open("test/Gravity.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let tick = exec_env
        .lookup_method(&mangle_method_name("Gravity", "Tick", "()V"))
        .unwrap();
    let set = exec_env
        .lookup_method(&mangle_method_name("Gravity", "SetPos", "(FF)V"))
        .unwrap();
    let set_vel = exec_env
        .lookup_method(&mangle_method_name("Gravity", "SetVel", "(FF)V"))
        .unwrap();
    let getx = exec_env
        .lookup_method(&mangle_method_name("Gravity", "GetX", "()F"))
        .unwrap();
    let gety = exec_env
        .lookup_method(&mangle_method_name("Gravity", "GetY", "()F"))
        .unwrap();
    let class = exec_env.lookup_class("Gravity").unwrap();
     let sqrt_extern_call = exec_env
        .lookup_method(&mangle_method_name("Gravity", "Sqrt", "(F)F"))
        .unwrap();
    exec_env.code_container.methods[sqrt_extern_call] =
        Some(Method::Invokable(Box::new(SqrtInvokable)));
    let obj = exec_env.new_obj(class);
    exec_env.call_method(set, &[Value::ObjectRef(obj),Value::Float(0.43),Value::Float(203.23)]).unwrap();
    exec_env.call_method(set_vel, &[Value::ObjectRef(obj),Value::Float(0.06125),Value::Float(0.0)]).unwrap();
    for i in 0..10_000{
        /*
        if i % 100 == 0{
            let x = exec_env.call_method(getx, &[Value::ObjectRef(obj)]).unwrap().as_float().unwrap();
            let y = exec_env.call_method(gety, &[Value::ObjectRef(obj)]).unwrap().as_float().unwrap();
            println!("({x},{y})");
        }*/
        //println!("Calling Tick!");
        exec_env.call_method(tick, &[Value::ObjectRef(obj)]).unwrap();
        //println!("Calling GetX!");
        
    }
    panic!();
}
#[test]
fn extends() {
    let mut file = std::fs::File::open("test/SuperClass.class").unwrap();
    let super_class = crate::importer::load_class(&mut file).unwrap();
    let mut file = std::fs::File::open("test/Extends.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(super_class);
    exec_env.load_class(class);
}
/*
#[test]
fn load_jar() {
    let mut file = std::fs::File::open("test/server.jar").unwrap();
    let classes = importer::load_jar(&mut file).unwrap();
    panic!();
}*/
