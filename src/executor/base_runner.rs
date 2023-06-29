use crate::BaseIR;
use crate::ExecCtx;
use crate::ExecException;
use crate::Value;
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
        //println!("op:{op:?}");
        match op {
            BaseIR::New(class_ref) => {
                let new_obj = ctx.new_obj(*class_ref);
                ctx.stack_push(Value::ObjectRef(new_obj));
            }
            BaseIR::ANewArray(_) => {
                let length = ctx.stack_pop().unwrap().as_int().unwrap() as usize;
                //let new_obj = ctx.new_obj(*class_ref);
                let new_arr = ctx.new_array(Value::ObjectRef(0), length);
                ctx.stack_push(Value::ObjectRef(new_arr));
            }
            BaseIR::ZNewArray => {
                let length = ctx.stack_pop().unwrap().as_int().unwrap() as usize;
                //let new_obj = ctx.new_obj(*class_ref);
                let new_arr = ctx.new_array(Value::Bool(false), length);
                ctx.stack_push(Value::ObjectRef(new_arr));
            }
            BaseIR::ArrayLength => {
                let arr = ctx.stack_pop().unwrap().as_objref().unwrap() as usize;
                ctx.stack_push(Value::Int(ctx.get_array_length(arr) as i32));
            }
            BaseIR::Dup => {
                let a: Value = ctx.stack_pop().unwrap().clone();
                ctx.stack_push(a);
                ctx.stack_push(a);
            }
            BaseIR::IConst(value) => ctx.stack_push(Value::Int(*value)),
            BaseIR::AConst(value) => ctx.stack_push(Value::ObjectRef(*value)),
            BaseIR::ILoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::FLoad(index) => ctx.stack_push(ctx.get_local(*index).unwrap().clone()),
            BaseIR::ALoad(index) => {
                let local = ctx.get_local(*index).unwrap().clone();
                assert_ne!(local, Value::Void, "Loading local at {index} yelded Void!");
                ctx.stack_push(local)
            }
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
            BaseIR::APutField(id) => {
                let val = ctx.stack_pop().unwrap();
                let obj_ref = ctx.stack_pop().unwrap();
                let obj_ref = obj_ref
                    .as_objref()
                    .expect(&format!("Expected object reference, got {obj_ref:?}!"));
                ctx.put_field(obj_ref, *id, val);
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
            BaseIR::AGetField(id) => {
                let obj_ref = ctx.stack_pop().unwrap().as_objref().unwrap();
                let value = ctx.get_field(obj_ref, *id).unwrap();
                ctx.stack_push(value);
            }
            BaseIR::AGetStatic(index) => {
                ctx.stack_push(ctx.get_static(*index));
            }
            BaseIR::APutStatic(index) => {
                let val = ctx.stack_pop().unwrap();
                ctx.put_static(*index, val);
            }
            BaseIR::BAStore => {
                let value = ctx.stack_pop().unwrap();
                let index = ctx.stack_pop().unwrap().as_int().unwrap();
                let array_ref = ctx.stack_pop().unwrap().as_objref().unwrap();
                ctx.set_array_at(array_ref, index as usize, value);
            }
            BaseIR::BALoad => {
                let index = ctx.stack_pop().unwrap().as_int().unwrap();
                let array_ref = ctx.stack_pop().unwrap().as_objref().unwrap();
                let value = ctx.get_array_at(array_ref, index as usize);
                ctx.stack_push(value);
            }
            BaseIR::IReturn | BaseIR::FReturn | BaseIR::AReturn => {
                return Ok(ctx.stack_pop().unwrap());
            }
            BaseIR::Return => {
                return Ok(Value::Void);
            }
            BaseIR::InvokeStatic(method_id, argc) => {
                let mut args: Box<[Value]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                // Hack
                let args = args;
                let res: Value = ctx.invoke_method(&args, *method_id)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            BaseIR::InvokeSpecial(method_id, argc) => {
                println!("method_id:{method_id}");
                let mut args: Box<[Value]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                // Hack
                let args = args;
                let res: Value = ctx.invoke_method(&args, *method_id)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            BaseIR::IInc(variable, inc) => {
                let prev = ctx.get_local(*variable).unwrap();
                let next = prev.as_int().unwrap() + *inc as i32;
                ctx.set_local(*variable, Value::Int(next));
            }
            BaseIR::InvokeVirtual(method_id, argc) => {
                let mut args: Box<[Value]> = (0..*argc).map(|_| ctx.stack_pop().unwrap()).collect();
                args.reverse();
                let obj_ref = args[0].as_objref().unwrap();
                let virtual_method = ctx.get_virtual(obj_ref, *method_id).unwrap();
                //todo!("virtual:{virtual_method:?}");
                let res: Value = ctx.invoke_method(&args, virtual_method)?;
                if let Value::Void = res {
                } else {
                    ctx.stack_push(res)
                };
            }
            BaseIR::IfIGreterEqual(jump_index) => {
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                let b = ctx.stack_pop().unwrap().as_int().unwrap();
                //println!("{a} <={b}?{}", a <= b );
                if a <= b {
                    op_index = *jump_index;
                    continue;
                }
            }
            BaseIR::IfZero(jump_index) => {
                let a = ctx.stack_pop().unwrap().as_int().unwrap();
                if a == 0 {
                    op_index = *jump_index;
                    continue;
                }
            }
            BaseIR::GoTo(jump_index) => {
                op_index = *jump_index;
                continue;
            }
            _ => todo!("Can't execute {op:?} yet!"),
        }
        op_index += 1;
    }
}
