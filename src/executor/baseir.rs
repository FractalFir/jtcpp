use super::UnmetDependency;
use crate::executor::fatops::FatOp;
use crate::ExecEnv;
use crate::{CodeContainer, EnvMemory, ExecException, Value};
#[derive(Debug)]
pub(crate) enum BaseIR {
    Dup,
    IConst(i32),
    LConst(i32),
    AConst(usize),
    FConst(f32),
    DConst(f64),
    ALoad(u8),
    FLoad(u8),
    DLoad(u8),
    ILoad(u8),
    IStore(u8),
    FStore(u8),
    IAdd,
    FAdd,
    FSub,
    IMul,
    FMul,
    ISub,
    IRem,
    IDiv,
    InvokeSpecial(usize, u8),
    InvokeStatic(usize, u8),
    InvokeVirtual(usize, u8),
    ZGetStatic(usize),
    BGetStatic(usize),
    SGetStatic(usize),
    IGetStatic(usize),
    LGetStatic(usize),
    FGetStatic(usize),
    DGetStatic(usize),
    OGetStatic(usize),
    CGetStatic(usize),
    ZGetField(usize),
    BGetField(usize),
    SGetField(usize),
    IGetField(usize),
    LGetField(usize),
    FGetField(usize),
    DGetField(usize),
    OGetField(usize),
    CGetField(usize),
    ZPutField(usize),
    BPutField(usize),
    SPutField(usize),
    IPutField(usize),
    LPutField(usize),
    FPutField(usize),
    DPutField(usize),
    OPutField(usize),
    CPutField(usize),
    Return,
    IReturn,
    FReturn,
}
pub(crate) fn into_base(
    fat: &[FatOp],
    exec_env: &mut ExecEnv,
) -> Result<Box<[BaseIR]>, UnmetDependency> {
    let mut newops = Vec::with_capacity(fat.len());
    for op in fat {
        let newop = match op {
            FatOp::IConst(int) => BaseIR::IConst(*int),
            FatOp::ISub => BaseIR::ISub,
            FatOp::FSub => BaseIR::FSub,
            FatOp::IAdd => BaseIR::IAdd,
            FatOp::FAdd => BaseIR::FAdd,
            FatOp::IMul => BaseIR::IMul,
            FatOp::FMul => BaseIR::FMul,
            FatOp::IDiv => BaseIR::IDiv,
            FatOp::IRem => BaseIR::IRem,
            FatOp::ALoad(index) => BaseIR::ALoad(*index),
            FatOp::ILoad(index) => BaseIR::ILoad(*index),
            FatOp::IStore(index) => BaseIR::IStore(*index),
            FatOp::FStore(index) => BaseIR::FStore(*index),
            FatOp::FLoad(index) => BaseIR::FLoad(*index),
            FatOp::InvokeSpecial(mangled, args) => {
                let method_id = exec_env.code_container.lookup_or_insert_method(&mangled);
                BaseIR::InvokeSpecial(method_id, *args)
            }
            FatOp::InvokeStatic(mangled, args) => {
                let method_id = exec_env.code_container.lookup_or_insert_method(&mangled);
                BaseIR::InvokeStatic(method_id, *args)
            }
            FatOp::FGetField(class_name,field_name)=>{
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let (field_id,_ftype) = exec_env.code_container.classes[class_id].get_field(field_name).unwrap();
                BaseIR::FGetField(field_id)
            },
            FatOp::FPutField(class_name,field_name)=>{
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let (field_id,_ftype) = exec_env.code_container.classes[class_id].get_field(field_name).unwrap();
                BaseIR::FPutField(field_id)
            },
            FatOp::Return => BaseIR::Return,
            FatOp::IReturn => BaseIR::IReturn,
            FatOp::FReturn => BaseIR::FReturn,
            FatOp::Dup => BaseIR::Dup,
            _ => todo!("Can't convert op {op:?} to base IR"),
        };
        newops.push(newop);
    }
    Ok(newops.into())
}
pub(crate) fn call(
    ops: &[BaseIR],
    args: &[Value],
    memory: &mut EnvMemory,
    code_container: &CodeContainer,
) -> Result<Value, ExecException> {
    let mut op_index = 0;
    let mut stack: Vec<Value> = Vec::with_capacity(100);
    let mut locals: Vec<_> = args.into();
    //TODO: get actual number!
    let max_locals = 100;
    while locals.len() < max_locals as usize {
        locals.push(Value::Void);
    }
    loop {
        let op = &ops[op_index];
        println!("op:{op:?} stack:{stack:?}");
        match op {
            BaseIR::Dup=> {
                let a = stack.pop().unwrap();
                stack.push(a);
                stack.push(a);
            }
            BaseIR::IConst(value) => stack.push(Value::Int(*value)),
            BaseIR::ILoad(index) => stack.push(locals[*index as usize].clone()),
            BaseIR::FLoad(index) => stack.push(locals[*index as usize].clone()),
            BaseIR::ALoad(index) => stack.push(locals[*index as usize].clone()),
            BaseIR::IStore(index) => {
                let a = stack.pop().unwrap();
                locals[*index as usize] = a.clone();
            }
            BaseIR::FStore(index) => {
                let a = stack.pop().unwrap();
                locals[*index as usize] = a.clone();
            }
            BaseIR::IAdd => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a + b));
            }
            BaseIR::FAdd => {
                let a = stack.pop().unwrap().as_float().unwrap();
                let b = stack.pop().unwrap().as_float().unwrap();
                stack.push(Value::Float(a + b));
            }
            BaseIR::ISub => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a - b));
            }
            BaseIR::FSub => {
                let a = stack.pop().unwrap().as_float().unwrap();
                let b = stack.pop().unwrap().as_float().unwrap();
                stack.push(Value::Float(a - b));
            }
            BaseIR::IMul => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a * b));
            }
            BaseIR::FMul => {
                let a = stack.pop().unwrap().as_float().unwrap();
                let b = stack.pop().unwrap().as_float().unwrap();
                stack.push(Value::Float(a * b));
            }
            BaseIR::IDiv => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a / b));
            }
            BaseIR::IRem => {
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a % b));
            }
            BaseIR::FPutField(id) => {
                let val = stack.pop().unwrap();
                let obj_ref = stack.pop().unwrap();
                println!("{obj_ref:?} val:{val:?}");
                let obj_ref = obj_ref.as_objref().unwrap();
                let obj = &mut memory.objects[obj_ref];
                obj.set_field(*id,val);
            }
            BaseIR::FGetField(id) => {
                let obj_ref = stack.pop().unwrap().as_objref().unwrap();
                let obj = &memory.objects[obj_ref];
                let value = obj.get_field(*id).unwrap();
                println!("getfield val:{value:?}");
                stack.push(value);
            }
            BaseIR::IReturn | BaseIR::FReturn => {
                return Ok(stack.pop().unwrap());
            }
            BaseIR::Return => {
                return Ok(Value::Void);
            }
            BaseIR::InvokeStatic(index, argc) => {
                let method = code_container.methods[*index as usize].as_ref().unwrap();
                let args: Box<[_]> = (0..*argc).map(|_| stack.pop().unwrap()).collect();
                let res = method.call(&args, memory, code_container)?;
                if let Value::Void = res {
                } else {
                    stack.push(res)
                };
            }
            BaseIR::InvokeSpecial(index, argc) => {
                let method = code_container.methods[*index as usize].as_ref().unwrap();
                let args: Box<[_]> = (0..*argc).map(|_| stack.pop().unwrap()).collect();
                let res = method.call(&args, memory, code_container)?;
                if let Value::Void = res {
                } else {
                    stack.push(res)
                };
            }
            _ => todo!("Can't execute {op:?} yet!"),
        }
        op_index += 1;
    }
}
