use crate::executor::fatclass::FatClass;
use crate::executor::FieldType;
use crate::ClassRef;
use crate::ExecEnv;
use crate::Value;
const STDIO_INSERTION_MARKER: &str = "STDIO Inserted!";
const CORE_INSERTION_MARKER: &str = "Core Inserted!";
struct STDIO_PRINTLN_IMPL;
use crate::ExecCtx;
use crate::ExecException;
fn insert_exceptions(exec_env: &mut ExecEnv) {
    //getStackTrace
    let mut throwable = FatClass::new("java/lang/Throwable", "java/lang/Object");
    throwable.add_virtual("getStackTrace()[Ljava/lang/StackTraceElement;","java/lang/Throwable::getStackTrace()[Ljava/lang/StackTraceElement;");
    throwable.add_virtual("addSuppressed(Ljava/lang/Throwable;)V","java/lang/Throwable::addSuppressed(Ljava/lang/Throwable;)V");
    exec_env.insert_class(throwable);
    exec_env.insert_class(FatClass::new("java/lang/Exception", "java/lang/Throwable"));
    exec_env.insert_class(FatClass::new(
        "java/lang/RuntimeException",
        "java/lang/Exception",
    ));
    exec_env.insert_class(FatClass::new(
        "java/lang/IllegalStateException",
        "java/lang/RuntimeException",
    ));
}
fn insert_string_methods(exec_env: &mut ExecEnv) {

    //str::encode_utf16
    //exec_env.insert_class(FatClass::new("java/lang/Record","java/lang/Object"));
}
fn insert_record(exec_env: &mut ExecEnv) {
    exec_env.insert_class(FatClass::new("java/lang/Record", "java/lang/Object"));
}
impl crate::Invokable for STDIO_PRINTLN_IMPL {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException> {
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
    exec_env.insert_class(FatClass::new(STDIO_INSERTION_MARKER, "java/lang/Object"));
    let input_stream = FatClass::new("java/io/InputStream", "java/lang/Object");
    exec_env.insert_class(input_stream);
    let output_stream = FatClass::new("java/io/OutputStream", "java/lang/Object"); // OutputStream
    exec_env.insert_class(output_stream);
    let filter_output_stream = FatClass::new("java/io/FilterOutputStream", "java/io/OutputStream"); // OutputStream
    exec_env.insert_class(filter_output_stream);
    let mut print_stream = FatClass::new("java/io/PrintStream", "java/io/FilterOutputStream"); // OutputStrea
    print_stream.add_virtual(
        "println(Ljava/lang/String;)V",
        "java/io/PrintStream::println(Ljava/lang/String;)V",
    );
    print_stream.add_virtual(
        "println(Ljava/lang/Object;)V",
        "java/io/PrintStream::println(Ljava/lang/Object;)V",
    );
    let print_stream = exec_env.insert_class(print_stream);

    let mut stdout_stream = FatClass::new("jbi/Internal/io/out", "java/io/PrintStream"); // OutputStream
    const JBI_PRINTLN_IMPL_NAME: &str = "jbi/Internal/io/out::println(Ljava/lang/String;)V";
    stdout_stream.add_virtual("println(Ljava/lang/String;)V", JBI_PRINTLN_IMPL_NAME);
    let stdout_stream = exec_env.insert_class(stdout_stream).unwrap();
    let jbi_stdio_println_impl_method = exec_env.lookup_method(JBI_PRINTLN_IMPL_NAME).unwrap();
    exec_env.replace_method_with_extern(jbi_stdio_println_impl_method, STDIO_PRINTLN_IMPL);

    let mut system = FatClass::new("java/lang/System", "java/lang/Object"); // OutputStream
    system.add_static("out", FieldType::ObjectRef);
    system.add_static("err", FieldType::ObjectRef);
    let system: ClassRef = exec_env.insert_class(system).unwrap();
    let system_static_out_ref = exec_env.get_static_id(system, &"out").unwrap();
    let system_static_out_obj = Value::ObjectRef(exec_env.new_obj(stdout_stream));
    exec_env.set_static(system_static_out_ref, system_static_out_obj);
    //TODO: this is an iterface. How to handle interfaces? It is also generic.
    exec_env.insert_class(FatClass::new(
        "java/nio/file/attribute/FileAttribute",
        "java/lang/Object",
    ));
    // TODO: Handle generics
    exec_env.insert_class(FatClass::new(
        "java/util/AbstractCollection",
        "java/lang/Object",
    ));
    exec_env.insert_class(FatClass::new(
        "java/util/AbstractList",
        "java/util/AbstractCollection",
    ));
    exec_env.insert_class(FatClass::new(
        "java/util/ArrayList",
        "java/util/AbstractList",
    ));
    //todo!();
}
pub(crate) fn insert_core(exec_env: &mut ExecEnv) {
    if let Some(_) = exec_env.lookup_class(CORE_INSERTION_MARKER) {
        return;
    }
    //exec_env.insert_class(FatClass::new("java/net/URLClassLoader","java/lang/Object"));
    exec_env.insert_class(FatClass::new(CORE_INSERTION_MARKER, "java/lang/Object"));
    let mut class_loader = FatClass::new("java/lang/ClassLoader", "java/lang/Object");
    class_loader.add_virtual(
        "getParent()Ljava/lang/ClassLoader;",
        "java/lang/ClassLoader::getParent()Ljava/lang/ClassLoader;",
    );
    exec_env.insert_class(class_loader);
    insert_record(exec_env);
    insert_string_methods(exec_env);
    insert_exceptions(exec_env);
}
pub(crate) fn insert_all(exec_env: &mut ExecEnv) {
    insert_core(exec_env);
    insert_stdio(exec_env);
}
