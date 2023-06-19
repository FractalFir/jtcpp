pub(crate) type IString = Box<str>;
use crate::importer::OpCode;
use std::collections::HashMap;
mod importer;
struct Class {}
impl Class{
    fn empty()->Self{
        Self{}
    }
}
enum Method {
    RawOps { ops: Box<[OpCode]>, max_locals: u16,argc:u8 },
}
fn call_raw_ops(
    ops: &[OpCode],
    max_locals: u16,
    args: &[Value],
    memory: &mut EnvMemory,
    code_container: &CodeContainer,
) -> Result<Value, ExecException> {
    let mut stack: Vec<Value> = Vec::new();
    let mut locals: Vec<_> = args.into();
    while locals.len() < max_locals as usize {
        locals.push(Value::Void);
    }
    let mut op_index = 0;
    println!("ops:{ops:?}\n\n");
    loop {
        //println!("stack:{stack:?}");
        let curr_op = &ops[op_index];
        match curr_op {
            OpCode::ILoad(index) => stack.push(locals[*index as usize].clone()),
            OpCode::IStore(index) => locals[*index as usize] = stack.pop().unwrap(),
            OpCode::IReturn => {
                return Ok(stack.pop().unwrap());
            }
            OpCode::IAdd => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a + b));
            }
            OpCode::IMul => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a * b));
            }
            OpCode::IDiv => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a / b));
            }
            OpCode::IRem => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a % b));
            }
            OpCode::ISub => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a - b));
            }
            OpCode::InvokeStatic(index) => {
                //let arg
                let method = code_container.methods[*index as usize].as_ref().unwrap();
                let argc = method.argument_count();
                //TODO: Ensure proper arg order!
                let args:Box<[_]> = (0..argc).map(|_|{stack.pop().unwrap()}).collect();
                let res = method.call(&args,memory,code_container)?;
                if let Value::Void = res{}
                else {stack.push(res)};
            }
            _ => todo!("Can't handle opcode {curr_op:?}"),
        }
        op_index += 1;
    }
    panic!("Unreachable condition reached!");
}
impl Method {
    fn argument_count(&self)->usize{
        match self {
            Self::RawOps { ops, max_locals,argc } => *argc as usize,
        }
    }
    fn call(
        &self,
        args: &[Value],
        memory: &mut EnvMemory,
        code_container: &CodeContainer,
    ) -> Result<Value, ExecException> {
        match self {
            Self::RawOps { ops, max_locals, argc} => {
                call_raw_ops(&ops, *max_locals, args, memory, code_container)
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum Value {
    Void,
    Int(i32),
    ObjectRef(i32),
}
impl Value {
    fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(a) => Some(*a),
            _ => None,
        }
    }
}
enum Object {
    Object { values: Box<[Value]> },
    String(IString),
}
struct EnvMemory {
    objects: Vec<Object>,
    statics: Vec<Value>,
}
impl EnvMemory {
    fn new() -> Self {
        Self {
            objects: Vec::with_capacity(0x100),
            statics: Vec::with_capacity(0x100),
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
    fn lookup_or_insert_class(&mut self, name: &str) -> usize {
        *self
            .class_names
            .entry(name.to_owned().into_boxed_str())
            .or_insert_with(|| {
                self.classes.push(Class::empty());
                self.classes.len() - 1
            })
    }
    fn set_or_replace_class(&mut self, name: &str, class: Class) -> usize {
        let idx = self.lookup_or_insert_class(name);
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
        let classes = vec![object_class];
        let class_names = HashMap::with_capacity(0x100);
        let method_names = HashMap::with_capacity(0x100);
        Self {
            methods,
            classes,
            class_names,
            method_names,
        }
    }
    //fn set_meth
}
struct ExecEnv {
    code_container: CodeContainer,
    env_mem: EnvMemory,
    //objects:Vec<Option<Object>>
}
#[test]
fn arg_counter(){
    assert_eq!(method_desc_to_argc("()I"),0);
    assert_eq!(method_desc_to_argc("(I)I"),1);
    assert_eq!(method_desc_to_argc("(IL)I"),2);
    assert_eq!(method_desc_to_argc("(ILF)I"),3);
    assert_eq!(method_desc_to_argc("(ILF)"),3);
}
fn method_desc_to_argc(desc:&str)->u8{
    assert_eq!(desc.chars().nth(0),Some('('));
    let mut char_beg = 0;
    let mut char_end = 0;
    for (index,character) in desc.chars().enumerate(){
        if character == '('{
            assert_eq!(char_beg,0);
            char_beg = index;
        }
        else if character == ')'{
            assert_eq!(char_end,0);
            char_end = index;
        }
    }
    (char_end - char_beg - 1) as u8
}
fn mangle_method_name(class: &str, method: &str,desc:&str) -> IString {
    format!("{class}::{method}{desc}").into_boxed_str()
}
impl ExecEnv {
    fn lookup_or_insert_static(&mut self, class:usize, static_field:&str)-> usize{
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
        let mut bytecode = bytecode.to_owned();
        for mut op in bytecode.iter_mut() {
            match op {
                OpCode::GetStatic(index) => {
                    let lookup_filed_ref = class.lookup_filed_ref(*index).unwrap();
                    let class = class.lookup_class(lookup_filed_ref.0).unwrap();
                    //let class_index
                    //println!("class:{class}");
                    //let const_string = class.lookup_const_string(*index as usize).unwrap();
                    //println!("Const string:{const_string}");
                    //self.env_mem.push
                }
                OpCode::InvokeStatic(index) => {
                    let (method_class, nametype) = class.lookup_method_ref(*index).unwrap();
                    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
                    let method_class = class.lookup_class(method_class).unwrap();
                    let name = class.lookup_utf8(name).unwrap();
                    let descriptor = class.lookup_utf8(descriptor).unwrap();
                    let mangled = mangle_method_name(method_class, name,descriptor);
                    let method_id = self.code_container.lookup_or_insert_method(&mangled);
                    *op = OpCode::InvokeStatic(method_id as u16);
                }
                _ => (),
            }
        }
        let max_locals = method.max_locals().unwrap();
        let (name,  descriptor) = (method.name_index(), method.descriptor_index());
        let method_class = class.lookup_class(class.this_class()).unwrap();
        let name = class.lookup_utf8(name).unwrap();
        let descriptor = class.lookup_utf8(descriptor).unwrap();
        let mangled = mangle_method_name(method_class, name,descriptor);
        let method_id = self.code_container.lookup_or_insert_method(&mangled);
        let argc = method_desc_to_argc(&descriptor);
        println!("mangled:{mangled}");
        self.code_container.methods[method_id] = Some(Method::RawOps {
            ops: bytecode.into(),
            max_locals,
            argc,
        });
    }
    pub(crate) fn load_class(&mut self, class: crate::importer::ImportedJavaClass) {
        for field in class.fields(){
            todo!("Can't load fields yet!");
        }
        for method in class.methods() {
            self.load_method(method, &class);
        }
    }
    pub(crate) fn lookup_method(&self,method_name:&str)->Option<usize>{
        self.code_container.method_names.get(method_name).copied()
    }
    pub(crate) fn call_method(
        &mut self,
        method_id: usize,
        args: &[Value],
    ) -> Result<Value, ExecException> {
        let mut args:Vec<_> = args.into();
        args.reverse();
        let method = self.code_container.methods.get(method_id);
        method
            .ok_or(ExecException::MethodNotFound)?
            .as_ref()
            .ok_or(ExecException::MethodNotFound)?
            .call(&args, &mut self.env_mem, &self.code_container)
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
    let identity = exec_env.lookup_method(&mangle_method_name("Identity","Identity","(I)I")).unwrap();
    for a in 0..1000 {
        assert_eq!(
            exec_env.call_method(identity,&[Value::Int(a)]).unwrap(),
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
    let add = exec_env.lookup_method(&mangle_method_name("BasicArthm","Add","(II)I")).unwrap();
    let sub = exec_env.lookup_method(&mangle_method_name("BasicArthm","Sub","(II)I")).unwrap();
    let mul = exec_env.lookup_method(&mangle_method_name("BasicArthm","Mul","(II)I")).unwrap();
    let div = exec_env.lookup_method(&mangle_method_name("BasicArthm","Div","(II)I")).unwrap();
    let rem = exec_env.lookup_method(&mangle_method_name("BasicArthm","Mod","(II)I")).unwrap();
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env.call_method( add,&[Value::Int(a), Value::Int(b)]).unwrap(),
                Value::Int(a + b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env.call_method( sub,&[Value::Int(a), Value::Int(b)]).unwrap(),
                Value::Int(a - b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env.call_method(mul,&[Value::Int(a), Value::Int(b)]).unwrap(),
                Value::Int(a * b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env.call_method(div,&[Value::Int(a), Value::Int(b)]).unwrap(),
                Value::Int(a / b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env.call_method(rem,&[Value::Int(a), Value::Int(b)]).unwrap(),
                Value::Int(a % b)
            );
        }
    }
}
#[test]
fn exec_call() {
    let mut file = std::fs::File::open("test/Calls.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let sqr_mag = exec_env.lookup_method(&mangle_method_name("Calls","SqrMag","(III)I")).unwrap();
    let first = exec_env.lookup_method(&mangle_method_name("Calls","ReturnFirst","(IIIII)I")).unwrap();
    let second = exec_env.lookup_method(&mangle_method_name("Calls","ReturnSecond","(IIIII)I")).unwrap();
    let last = exec_env.lookup_method(&mangle_method_name("Calls","ReturnLast","(IIIII)I")).unwrap();
    let first_bck = exec_env.lookup_method(&mangle_method_name("Calls","ReturnFirst","(IIIII)I")).unwrap();
     assert_eq!(exec_env.call_method(first_bck,&[Value::Int(1), Value::Int(2),Value::Int(3), Value::Int(4),Value::Int(5)]).unwrap(),Value::Int(1));
    assert_eq!(exec_env.call_method(first,&[Value::Int(1), Value::Int(2),Value::Int(3), Value::Int(4),Value::Int(5)]).unwrap(),Value::Int(1));
     assert_eq!(exec_env.call_method(second,&[Value::Int(1), Value::Int(2),Value::Int(3), Value::Int(4),Value::Int(5)]).unwrap(),Value::Int(2));
     assert_eq!(exec_env.call_method(last,&[Value::Int(1), Value::Int(2),Value::Int(3), Value::Int(4),Value::Int(5)]).unwrap(),Value::Int(5));
    for a in 0..1000 {
        exec_env.call_method(sqr_mag,&[Value::Int(a), Value::Int(7),Value::Int(8)]).unwrap();
    };
}

#[test]
fn exec_hw() {
    let mut file = std::fs::File::open("test/HelloWorld.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    
    let hw = exec_env.lookup_method(&mangle_method_name("HelloWorld","main","([Ljava/lang/String;)V")).unwrap();
    exec_env.call_method(hw,&[]).unwrap();
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
