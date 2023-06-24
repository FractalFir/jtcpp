use crate::ExecEnv;
use crate::executor::fatclass::FatClass;
use crate::executor::FieldType;
use crate::ClassRef;
use crate::Value;
const STDIO_INSERTION_MARKER: &str = "STDIO Inserted!";
struct STDIO_PRINTLN_IMPL;
use crate::ExecCtx;
use crate::ExecException;
impl crate::Invokable for STDIO_PRINTLN_IMPL{
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException>{
        let arg = ctx.get_local(1).unwrap();
        let objref = arg.as_objref().unwrap();
        let string = ctx.to_string(objref).unwrap();
        println!("{string}");
        Ok(Value::Void)
    }
}
pub(crate) fn insert_stdio(exec_env: &mut ExecEnv) {
    if let Some(_) = exec_env.lookup_class(STDIO_INSERTION_MARKER) {
        return;
    }
    let output_stream = FatClass::new("java/io/OutputStream","java/lang/Object");// OutputStream 
    exec_env.insert_class(output_stream);
    let filter_output_stream = FatClass::new("java/io/FilterOutputStream","java/io/OutputStream");// OutputStream 
    exec_env.insert_class(filter_output_stream);
    let mut print_stream = FatClass::new("java/io/PrintStream","java/io/FilterOutputStream");// OutputStream 

    print_stream.add_virtual("println(Ljava/lang/String;)V","java/io/PrintStream::println(Ljava/lang/String;)V");
    let print_stream = exec_env.insert_class(print_stream);
    
    
    let mut stdout_stream = FatClass::new("jbi/Internal/io/out","java/io/PrintStream");// OutputStream 
    const JBI_PRINTLN_IMPL_NAME:&str = "jbi/Internal/io/out::println(Ljava/lang/String;)V";
    stdout_stream.add_virtual("println(Ljava/lang/String;)V",JBI_PRINTLN_IMPL_NAME);
    let mut stdout_stream = exec_env.insert_class(stdout_stream);
    let jbi_stdio_println_impl_method = exec_env.lookup_method(JBI_PRINTLN_IMPL_NAME).unwrap();
    exec_env.replace_method_with_extern(jbi_stdio_println_impl_method,STDIO_PRINTLN_IMPL);
    
    let mut system = FatClass::new("java/lang/System","java/lang/Object");// OutputStream 
    system.add_static("out",FieldType::ObjectRef);    
    let system:ClassRef = exec_env.insert_class(system);
    let system_static_out_ref = exec_env.get_static_id(system,&"out").unwrap();
    let system_static_out_obj = Value::ObjectRef(exec_env.new_obj(stdout_stream));
    exec_env.set_static(system_static_out_ref,system_static_out_obj);
    //todo!();
}
pub(crate) fn insert_all(exec_env: &mut ExecEnv) {
    insert_stdio(exec_env);
}
