use super::ExecCtx;
use super::UnmetDependency;
use crate::executor::fatops::FatOp;
use crate::ExecEnv;
use crate::ClassRef;
use crate::{CodeContainer, EnvMemory, ExecException, Value};
#[derive(Debug)]
pub(crate) enum BaseIR {
    Dup,
    Pop,
    AConst(usize),
    DConst(f64),
    FConst(f32),
    IConst(i32),
    LConst(i32),
    ALoad(u8),
    FLoad(u8),
    DLoad(u8),
    ILoad(u8),
    AStore(u8),
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
    FDiv,
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
    AGetStatic(usize),
    CGetStatic(usize),
    ZGetField(usize),
    BGetField(usize),
    SGetField(usize),
    IGetField(usize),
    LGetField(usize),
    FGetField(usize),
    DGetField(usize),
    AGetField(usize),
    CGetField(usize),
    ZPutField(usize),
    BPutField(usize),
    SPutField(usize),
    IPutField(usize),
    LPutField(usize),
    FPutField(usize),
    DPutField(usize),
    APutField(usize),
    CPutField(usize),
    Return,
    AReturn,
    IReturn,
    FReturn,
    F2D,
    D2F,
    New(ClassRef),
}
pub(crate) fn into_base(
    fat: &[FatOp],
    exec_env: &mut ExecEnv,
) -> Result<Box<[BaseIR]>, UnmetDependency> {
    //println!("Fat:{fat:?}");
    let mut newops = Vec::with_capacity(fat.len());
    for op in fat {
        let newop = match op {
            FatOp::FConst(float) => BaseIR::FConst(*float),
            FatOp::IConst(int) => BaseIR::IConst(*int),
            FatOp::ISub => BaseIR::ISub,
            FatOp::FSub => BaseIR::FSub,
            FatOp::IAdd => BaseIR::IAdd,
            FatOp::FAdd => BaseIR::FAdd,
            FatOp::IMul => BaseIR::IMul,
            FatOp::FMul => BaseIR::FMul,
            FatOp::IDiv => BaseIR::IDiv,
            FatOp::FDiv => BaseIR::FDiv,
            FatOp::IRem => BaseIR::IRem,
            FatOp::F2D => BaseIR::F2D,
            FatOp::D2F => BaseIR::D2F,
            FatOp::ALoad(index) => BaseIR::ALoad(*index),
            FatOp::ILoad(index) => BaseIR::ILoad(*index),
            FatOp::AStore(index) => BaseIR::AStore(*index),
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
            FatOp::InvokeVirtual(class_name,method,argc)=>{
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let virtual_index = exec_env.code_container.classes[class_id].lookup_virtual(method).unwrap();
                BaseIR::InvokeVirtual(virtual_index,*argc)
            }
            FatOp::FGetField(class_name, field_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let (field_id, _ftype) = exec_env.code_container.classes[class_id]
                    .get_field(field_name)
                    .unwrap();
                BaseIR::FGetField(field_id)
            }
            FatOp::AGetField(class_name, field_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let (field_id, _ftype) = exec_env.code_container.classes[class_id]
                    .get_field(field_name)
                    .unwrap();
                BaseIR::AGetField(field_id)
            }
            FatOp::FPutField(class_name, field_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let (field_id, _ftype) = exec_env.code_container.classes[class_id]
                    .get_field(field_name)
                    .unwrap();
                BaseIR::FPutField(field_id)
            }
            FatOp::APutField(class_name, field_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let (field_id, _ftype) = exec_env.code_container.classes[class_id]
                    .get_field(field_name)
                    .unwrap();
                BaseIR::APutField(field_id)
            }
            FatOp::IGetStatic(class_name, field_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let static_id = exec_env.code_container.classes[class_id]
                    .get_static(field_name)
                    .unwrap();
                BaseIR::IGetStatic(static_id)
            }
            FatOp::AGetStatic(class_name, field_name) => {
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                let static_id = exec_env.code_container.classes[class_id]
                    .get_static(field_name)
                    .unwrap();
                BaseIR::AGetStatic(static_id)
            }
            FatOp::New(class_name) =>{
                let class_id = exec_env.code_container.lookup_class(class_name);
                let class_id = if let Some(class_id) = class_id {
                    class_id
                } else {
                    return Err(UnmetDependency::NeedsClass(class_name.clone()));
                };
                BaseIR::New(class_id)
            },
            FatOp::SConst(string) => BaseIR::AConst(exec_env.const_string(string)),
            FatOp::Return => BaseIR::Return,
            FatOp::AReturn => BaseIR::AReturn,
            FatOp::IReturn => BaseIR::IReturn,
            FatOp::FReturn => BaseIR::FReturn,
            FatOp::Dup => BaseIR::Dup,
            FatOp::Pop => BaseIR::Pop,
            _ => todo!("Can't convert op {op:?} to base IR"),
        };
        //println!("Op:{op:?} new_op:{newop:?}");
        newops.push(newop);
    }
    Ok(newops.into())
}
pub(crate) fn call<'caller, 'env>(ops: &[BaseIR], mut ctx: ExecCtx) -> Result<Value, ExecException>
where
    'caller: 'env,
{
    let mut op_index = 0;
    //let mut stack: Vec<Value> = Vec::with_capacity(100);
    //let mut locals: Vec<_> = args.into();
    //TODO: get actual number!
    //let max_locals = 100;
    //while locals.len() < max_locals as usize {
    //locals.push(Value::Void);
    //}
    loop {
        let op = &ops[op_index];
        //println!("op:{op:?} stack:{stack:?}");
        match op {
            BaseIR::Dup => {
                let a: Value = ctx.stack_pop().unwrap().clone();
                ctx.stack_push(a);
                ctx.stack_push(a);
            }
            BaseIR::IConst(value) => ctx.stack_push(Value::Int(*value)),
            BaseIR::AConst(value) => ctx.stack_push(Value::ObjectRef(*value)),
            BaseIR::ILoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::FLoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::ALoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::IStore(index) => {
                let a = ctx.stack_pop().unwrap();
                ctx.set_local(*index, a.clone());
            }
            BaseIR::FStore(index) => {
                let a = ctx.stack_pop().unwrap();
                ctx.set_local(*index, a.clone());
            }
            BaseIR::IAdd => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a + b));
            }
            BaseIR::FAdd => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a + b));
            }
            BaseIR::ISub => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a - b));
            }
            BaseIR::FSub => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a - b));
            }
            BaseIR::IMul => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a * b));
            }
            BaseIR::FMul => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a * b));
            }
            BaseIR::IDiv => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a / b));
            }
            BaseIR::FDiv => {
                let b = ctx.stack_pop().unwrap().as_float().unwrap();
                let a = ctx.stack_pop().unwrap().as_float().unwrap();
                ctx.stack_push(Value::Float(a / b));
            }
            BaseIR::IRem => {
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                ctx.stack_push(Value::Int(a % b));
            }
            BaseIR::FPutField(id) => {
                let val = ctx.stack_pop().unwrap();
                let obj_ref = ctx.stack_pop().unwrap();
                let obj_ref = obj_ref.as_objref().unwrap();
                ctx.put_field(obj_ref, *id, val);
            }
            BaseIR::FGetField(id) => {
                let obj_ref = ctx.stack_pop().unwrap().as_objref().unwrap();
                let value = ctx.get_field(obj_ref, *id).unwrap();
                ctx.stack_push(value);
            }
            BaseIR::AGetStatic(index)=>{
                 ctx.stack_push(ctx.get_static(*index));
            }
            BaseIR::IReturn | BaseIR::FReturn => {
                return Ok(ctx.stack_pop().unwrap());
            }
            BaseIR::Return => {
                return Ok(Value::Void);
            }
            BaseIR::InvokeStatic(method_id, argc) => {
                let mut args: Box<[Value]> = (0..*argc)
                    .map(|_| ctx.stack_pop().unwrap().clone())
                    .collect();
                args.reverse();
                // Hack
                let args = args;
                let res: Value = ctx.invoke_method(&args, *method_id)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            } 
            BaseIR::InvokeVirtual(method_id,argc) => {
                let mut args: Box<[Value]> = (0..*argc)
                    .map(|_| ctx.stack_pop().unwrap().clone())
                    .collect();
                args.reverse();
                let obj_ref = args[0].as_objref().unwrap();
                let virtual_method = ctx.get_virtual(obj_ref,*method_id).unwrap();
                //todo!("virtual:{virtual_method:?}");
                let res: Value = ctx.invoke_method(&args, virtual_method)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            /*
            BaseIR::InvokeSpecial(index, argc) => {
                let method = code_container.methods[*index as usize].as_ref().unwrap();
                let mut args: Box<[_]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                let res = method.call(&args, memory, code_container)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }*/
            _ => todo!("Can't execute {op:?} yet!"),
        }
        op_index += 1;
    }
}
