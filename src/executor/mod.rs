pub(crate) mod baseir;
pub(crate) mod class;
pub(crate) mod fatclass;
pub(crate) mod fatops;
use crate::importer::ImportedJavaClass;
use core::ptr::NonNull;
use crate::{Value,IString};
use crate::ObjectRef;
#[derive(Debug)]
pub(crate) enum UnmetDependency {
    NeedsClass(IString),
}
#[derive(Debug,Clone)]
pub(crate) enum FieldType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    ObjectRef,
    Bool,
}
impl FieldType {
    fn default_value(&self) -> Value {
        match self {
            Self::Int => Value::Int(0),
            Self::Float => Value::Float(0.0),
            Self::ObjectRef => Value::ObjectRef(0),
            _ => todo!("Can't create default value of type {self:?}"),
        }
    }
}
pub(crate) fn field_descriptor_to_ftype(descriptor: u16, class: &ImportedJavaClass) -> FieldType {
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    match descriptor.chars().nth(0).unwrap() {
        'B' => FieldType::Byte,
        'C' => FieldType::Char,
        'D' => FieldType::Double,
        'F' => FieldType::Float,
        'I' => FieldType::Int,
        'J' => FieldType::Long,
        'L' | '[' => FieldType::ObjectRef,
        'S' => FieldType::Short,
        'Z' => FieldType::Bool,
        _ => panic!("Invalid field descriptor!\"{descriptor}\""),
    }
}
use smallvec::SmallVec;
use crate::{EnvMemory,CodeContainer,ExecException};
use core::cell::UnsafeCell;
//TODO: Make this safer(stop using NonNull, start using &). Lifetimes *realy* did not want to work with this.
pub(crate) struct ExecCtx<'env>
{
    caller:Option<NonNull<ExecCtx<'env>>>,
    args:NonNull<[Value]>,
    locals:usize,
    data:SmallVec::<[Value;12]>,
    memory: &'env UnsafeCell<EnvMemory>,
    code_container: &'env CodeContainer,
}
fn ref_to_cell<T>(ptr:&mut T)->&UnsafeCell<T>{
     unsafe { &*(ptr as *mut T as *const UnsafeCell<T>)}
}
//-> Result<Value, ExecException>
impl<'env> ExecCtx<'env>{
    fn get_local(&self,id:u8)->Value{
        unsafe{**self.args.as_ref().get(id as usize).get_or_insert_with(||{&self.data[id as usize - self.args.len()]})}
    }
    fn set_local(&mut self,id:u8,value:Value){
        let idx = id as usize - self.args.len();
        self.data[idx] = value;
    }
    fn get_static(&self,id:usize)->Value{
        unsafe{EnvMemory::get_static(self.memory.get(), id)}
    }
    fn stack_push(&mut self,value:Value){
        self.data.push(value);
    }
    fn stack_pop(&mut self)->Option<Value>{
        self.data.pop()
    }
    fn put_field(&mut self,objref:ObjectRef,field_id:usize,value:Value){
        unsafe{EnvMemory::set_field(self.memory.get(), objref,field_id,value)}
    }
    fn get_field(&mut self,objref:ObjectRef,field_id:usize)->Option<Value>{
        unsafe{EnvMemory::get_field(self.memory.get(), objref,field_id)}
    }
    pub(crate) fn new(memory: &'env mut EnvMemory,code_container: &'env CodeContainer,args:&'env [Value],locals:usize)->Self{
        let mut data = SmallVec::<[Value;12]>::new();
        for _ in 0..locals{
            data.push(Value::Void);
        }
        let memory = ref_to_cell(memory);
        Self{memory,code_container,data,locals,args:args.into(),caller:None}
    }
    fn call(&mut self,args:&[Value],locals:usize,callable:impl Fn(Self)-> Result<Value, ExecException>)-> Result<Value, ExecException>
    {
        let mut data = SmallVec::<[Value;12]>::new();
        for _ in 0..locals{
            data.push(Value::Void);
        }
        //let adr = std::ptr::addr_of!(*self);
        let call_arg:ExecCtx<'env> = Self{memory:self.memory,code_container:self.code_container,data,locals,args:args.into(),caller:Some(self.into())};
        callable(call_arg)
    }
    fn invoke_method<'caller>(&mut self,args:&'caller [Value],method_id:usize)->Result<Value, ExecException>
        {
        let method = self.code_container.methods.get(method_id)
            .ok_or(ExecException::MethodNotFound)?
            .as_ref()
            .ok_or(ExecException::MethodNotFound)?;
        let method = |ctx|{method.call(ctx)};
        self.call(args,10,method)
    }
    /*
    fn call(parrent:&Self,args:&[Value],locals:usize,stack:usize,callable:impl Fn(&Self)->Result<Value, ExecException>)->Result<Value, ExecException>{
        
    }*/
}
