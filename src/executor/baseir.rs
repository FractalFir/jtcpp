use crate::executor::fatops::FatOp;
use crate::ExecEnv;
use crate::{EnvMemory,CodeContainer,Value,ExecException};
#[derive(Debug)]
pub(crate) enum UnmetDependency{

}
#[derive(Debug)]
pub(crate) enum BaseIR{
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
    InvokeSpecial(usize,u8),
    InvokeStatic(usize,u8),
    InvokeVirtual(usize,u8),
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
pub(crate) fn into_base(fat:&[FatOp],exec_env:&mut ExecEnv)->Result<Box<[BaseIR]>,UnmetDependency>{
    let mut newops = Vec::with_capacity(fat.len());
    for op in fat{
        let newop = match op{
            FatOp::IConst(int) => BaseIR::IConst(*int),
            FatOp::ISub=>BaseIR::ISub,
            FatOp::FSub=>BaseIR::FSub,
            FatOp::IAdd=>BaseIR::IAdd,
            FatOp::FAdd=>BaseIR::FAdd,
            FatOp::IMul=>BaseIR::IMul,
            FatOp::FMul=>BaseIR::FMul,
            FatOp::IDiv=>BaseIR::IDiv,
            FatOp::IRem=>BaseIR::IRem,
            FatOp::ALoad(index) => BaseIR::ALoad(*index),
            FatOp::ILoad(index) => BaseIR::ILoad(*index),
            FatOp::IStore(index) => BaseIR::IStore(*index),
            FatOp::FStore(index) => BaseIR::FStore(*index),
            FatOp::FLoad(index) => BaseIR::FLoad(*index),
            FatOp::InvokeSpecial(mangled,args)=>{
                  let method_id = exec_env.code_container.lookup_or_insert_method(&mangled);
                  BaseIR::InvokeSpecial(method_id,*args)
            }
            FatOp::InvokeStatic(mangled,args)=>{
                  let method_id = exec_env.code_container.lookup_or_insert_method(&mangled);
                  BaseIR::InvokeStatic(method_id,*args)
            }
            FatOp::Return=>BaseIR::Return,
            FatOp::IReturn=>BaseIR::IReturn,
            FatOp::FReturn=>BaseIR::FReturn,
            _=>todo!("Can't convert op {op:?} to base IR")
        };
        newops.push(newop);
    }
    Ok(newops.into())
}
pub(crate) fn call(
        ops:&[BaseIR],
        args: &[Value],
        memory: &mut EnvMemory,
        code_container: &CodeContainer) -> Result<Value, ExecException> {
    let mut op_index = 0;
    let mut stack: Vec<Value> = Vec::with_capacity(100);
    let mut locals: Vec<_> = args.into();
    //TODO: get actual number!
    let max_locals = 100;
    while locals.len() < max_locals as usize {
        locals.push(Value::Void);
    }
    loop{
        let op = &ops[op_index];
        match op{
            BaseIR::IConst(value) => stack.push(Value::Int(*value)),
            BaseIR::ILoad(index) => stack.push(locals[*index as usize].clone()),
            BaseIR::IStore(index) => {
                let a = stack.pop().unwrap();
                locals[*index as usize] = a.clone();
            }
            BaseIR::IAdd =>{
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a + b));
            }
            BaseIR::ISub =>{
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a - b));
            }
            BaseIR::IMul =>{
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a * b));
            }
            BaseIR::IDiv =>{
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a / b));
            }
            BaseIR::IRem =>{
                let a = stack.pop().unwrap().as_int().unwrap();
                let b = stack.pop().unwrap().as_int().unwrap();
                stack.push(Value::Int(a % b));
            }
            BaseIR::IReturn | BaseIR::FReturn=> {
                return Ok(stack.pop().unwrap());
            }
            BaseIR::InvokeStatic(index,argc) => {
                let method = code_container.methods[*index as usize].as_ref().unwrap();
                let args: Box<[_]> = (0..*argc).map(|_| stack.pop().unwrap()).collect();
                let res = method.call(&args, memory, code_container)?;
                if let Value::Void = res {
                } else {
                    stack.push(res)
                };
            }
            BaseIR::InvokeSpecial(index,argc) => {
                let method = code_container.methods[*index as usize].as_ref().unwrap();
                let args: Box<[_]> = (0..*argc).map(|_| stack.pop().unwrap()).collect();
                let res = method.call(&args, memory, code_container)?;
                if let Value::Void = res {
                } else {
                    stack.push(res)
                };
            }
            _=>todo!("Can't execute {op:?} yet!"),
        }
        op_index += 1;
    }
}
