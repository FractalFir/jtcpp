use crate::ExecEnv;
use crate::executor::fatclass::FatClass;
use crate::executor::FieldType;
const STDIO_INSERTION_MARKER: &str = "STDIO Inserted!";
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
    exec_env.insert_class(print_stream);
    let mut stdout_stream = FatClass::new("jbi/Internal/io/out","java/io/PrintStream");// OutputStream 
    
    stdout_stream.add_virtual("println(Ljava/lang/String;)V","jbi/Internal/io/out::println(Ljava/lang/String;)V");
    exec_env.insert_class(stdout_stream);
    let mut system = FatClass::new("java/lang/System","java/lang/Object");// OutputStream 
    system.add_static("out",FieldType::ObjectRef);
    exec_env.insert_class(system);
    //todo!();
}
pub(crate) fn insert_all(exec_env: &mut ExecEnv) {
    insert_stdio(exec_env);
}
