pub(crate) type IString = Box<str>;
use crate::importer::OpCode;
use std::collections::HashMap;
mod importer;
struct Class {}
enum Method {
    RawOps { ops: Box<[OpCode]>, max_locals: u16 },
}
fn call_raw_ops(ops: &[OpCode], max_locals: u16, args: &[Value]) -> Value {
    let mut stack: Vec<Value> = Vec::new();
    let mut locals: Vec<_> = args.into();
    while locals.len() < max_locals as usize {
        locals.push(Value::Void);
    }
    let mut op_index = 0;
    loop {
        let curr_op = &ops[op_index];
        match curr_op {
            OpCode::ILoad(index) => stack.push(locals[*index as usize].clone()),
            OpCode::IStore(index) => locals[*index as usize] = stack.pop().unwrap(),
            OpCode::IReturn => {
                return stack.pop().unwrap();
            }
            OpCode::IAdd => {
                let b = stack.pop().unwrap().as_int().unwrap();
                let a = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a + b));
            }
            OpCode::IMul => {
                let b = stack.pop().unwrap().as_int().unwrap();
                let a = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a * b));
            }
            OpCode::IDiv => {
                let b = stack.pop().unwrap().as_int().unwrap();
                let a = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a / b));
            }
            OpCode::IRem => {
                let b = stack.pop().unwrap().as_int().unwrap();
                let a = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a % b));
            }
            OpCode::ISub => {
                let b = stack.pop().unwrap().as_int().unwrap();
                let a = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a - b));
            }
            _ => todo!("Can't handle opcode {curr_op:?}"),
        }
        op_index += 1;
    }
    panic!("Unreachable condition reached!");
}
impl Method {
    fn call(&self, args: &[Value]) -> Value {
        match self {
            Self::RawOps { ops, max_locals } => call_raw_ops(ops, *max_locals, args),
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
struct CodeContainer{
    classes: Vec<Option<Class>>,
    class_names: HashMap<String,usize>,
    methods: Vec<Option<Method>>,
}
impl CodeContainer{
    fn lookup_or_insert_class(&mut self,name:&str)->usize{
        *self.class_names.entry(name.to_owned()).or_insert_with(||{self.classes.push(None); self.classes.len()})
    }
    fn set_or_replace_class(&mut self, name:&str,class:Class)->usize{
        let idx = self.lookup_or_insert_class(name);
        self.classes[idx] = Some(class);
        idx
    }
    //fn set_meth
}
impl CodeContainer{
    fn new()->Self{
        let object_class = Class {};
        let methods = Vec::new();
        let classes = vec![Some(object_class)];
        let class_names = HashMap::with_capacity(0x100);
        Self{methods,classes,class_names}
    }
}
struct ExecEnv {
    code_container:CodeContainer,
    env_mem: EnvMemory,
    //objects:Vec<Option<Object>>
}
impl ExecEnv {
    pub fn new() -> Self {
        let env_mem = EnvMemory::new();
        let code_container = CodeContainer::new();
        //let objects = vec!
        Self {
            code_container,
            env_mem,
        }
    }
    pub(crate) fn load_class(&mut self, class: crate::importer::ImportedJavaClass) {
        for method in class.methods() {
            let bytecode = if let Some(bytecode) = method.bytecode() {
                bytecode
            } else {
                self.code_container.methods.push(None);
                continue;
            };
            for mut op in bytecode {
                match op {
                    OpCode::GetStatic(index) => {
                        let lookup_filed_ref = class.lookup_filed_ref(*index).unwrap();
                        let class = class.lookup_class(lookup_filed_ref.0).unwrap();
                        println!("class:{class}");
                        //let const_string = class.lookup_const_string(*index as usize).unwrap();
                        //println!("Const string:{const_string}");
                        //self.env_mem.push
                    }
                    _ => (),
                }
            }
            let max_locals = method.max_locals().unwrap();
            self.code_container.methods.push(Some(Method::RawOps {
                ops: bytecode.into(),
                max_locals,
            }));
        }
    }
}
#[test]
fn exec_identity() {
    let mut file = std::fs::File::open("test/Identity.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    for a in 0..1000 {
        assert_eq!(
            exec_env.code_container.methods[1].as_ref().unwrap().call(&[Value::Int(a)]),
            Value::Int(a)
        );
    }
}
#[test]
fn exec_hw() {
    let mut file = std::fs::File::open("test/HelloWorld.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    for a in 0..1000 {
        exec_env.code_container.methods[1].as_ref().unwrap().call(&[]);
    }
}
#[test]
fn basic_arthm() {
    let mut file = std::fs::File::open("test/BasicArthm.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env.code_container.methods[1]
                    .as_ref()
                    .unwrap()
                    .call(&[Value::Int(a), Value::Int(b)]),
                Value::Int(a + b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env.code_container.methods[2]
                    .as_ref()
                    .unwrap()
                    .call(&[Value::Int(a), Value::Int(b)]),
                Value::Int(a - b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env.code_container.methods[3]
                    .as_ref()
                    .unwrap()
                    .call(&[Value::Int(a), Value::Int(b)]),
                Value::Int(a * b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env.code_container.methods[4]
                    .as_ref()
                    .unwrap()
                    .call(&[Value::Int(a), Value::Int(b)]),
                Value::Int(a / b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env.code_container.methods[5]
                    .as_ref()
                    .unwrap()
                    .call(&[Value::Int(a), Value::Int(b)]),
                Value::Int(a % b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            let sum = a + b;
            let mul = a * b;
            let dif = sum - mul;
            let val = ((dif % sum) + mul) / mul;
            assert_eq!(
                exec_env.code_container.methods[6]
                    .as_ref()
                    .unwrap()
                    .call(&[Value::Int(a), Value::Int(b)]),
                Value::Int(val)
            );
        }
    }
}
