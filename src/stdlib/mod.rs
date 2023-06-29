use crate::executor::fatclass::FatClass;
use crate::executor::FieldType;
use crate::ClassRef;
use crate::ExecEnv;
use crate::Value;
use crate::add_virtual;
const STDIO_INSERTION_MARKER: &str = "STDIO Inserted!";
const CORE_INSERTION_MARKER: &str = "Core Inserted!";
struct StdoutPrintlnString;
struct StdoutPrintlnObject;
struct StdoutPrintlnInt;
use crate::ExecCtx;
use crate::ExecException;
fn insert_exceptions(exec_env: &mut ExecEnv) {
    //getStackTrace
    let mut throwable = FatClass::new("java/lang/Throwable", "java/lang/Object");
    add_virtual!(throwable,"getStackTrace()[Ljava/lang/StackTraceElement;","java/lang/Throwable");
    add_virtual!(throwable,"addSuppressed(Ljava/lang/Throwable;)V","java/lang/Throwable");
    add_virtual!(throwable,"printStackTrace(Ljava/io/PrintStream;)V","java/lang/Throwable");
    add_virtual!(throwable,"getSuppressed()[Ljava/lang/Throwable;","java/lang/Throwable");
    add_virtual!(throwable,"getMessage()Ljava/lang/String;","java/lang/Throwable");
    add_virtual!(throwable,"printStackTrace()V","java/lang/Throwable");
    add_virtual!(throwable,"getCause()Ljava/lang/Throwable;","java/lang/Throwable");
    add_virtual!(throwable,"printStackTrace(Ljava/io/PrintWriter;)V","java/lang/Throwable");
    add_virtual!(throwable,"initCause(Ljava/lang/Throwable;)Ljava/lang/Throwable;","java/lang/Throwable");
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
fn insert_string_methods(_exec_env: &mut ExecEnv) {

    //str::encode_utf16
    //exec_env.insert_class(FatClass::new("java/lang/Record","java/lang/Object"));
}
fn insert_record(exec_env: &mut ExecEnv) {
    exec_env.insert_class(FatClass::new("java/lang/Record", "java/lang/Object"));
}
impl crate::Invokable for StdoutPrintlnString {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException> {
        let arg = ctx.get_local(1).unwrap();
        let objref = arg.as_objref().unwrap();
        let string = ctx.to_string(objref).unwrap();
        println!("{string}");
        Ok(Value::Void)
    }
}
impl crate::Invokable for StdoutPrintlnInt {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException> {
        let arg = ctx.get_local(1).unwrap();
        let int = arg.as_int().unwrap();
        println!("{int}");
        Ok(Value::Void)
    }
}
pub(crate) fn insert_path(exec_env: &mut ExecEnv) {
    let path = FatClass::new(
        "java/nio/file/Path",
        "java/lang/Object",
    );
    //add_virtual!(path,"","java/nio/file/Path");
    exec_env.insert_class(path);
}
pub(crate) fn insert_proxy(exec_env: &mut ExecEnv) {
    //panic!();
    let proxy = FatClass::new(
        "java/net/Proxy",
        "java/lang/Object",
    );
    //add_virtual!(path,"","java/net/Proxy");
    exec_env.insert_class(proxy);
}
pub(crate) fn insert_file(exec_env: &mut ExecEnv) {
    let mut file = FatClass::new(
        "java/io/File",
        "java/lang/Object",
    );
    add_virtual!(file,"getPath()Ljava/lang/String;","java/io/File");
    add_virtual!(file,"canExecute()Z","java/io/File");
    add_virtual!(file,"canWrite()Z","java/io/File");
    add_virtual!(file,"canRead()Z","java/io/File");
    add_virtual!(file,"toPath()Ljava/nio/file/Path;","java/io/File");
    add_virtual!(file,"getAbsoluteFile()Ljava/io/File;","java/io/File");
    add_virtual!(file,"listFiles()[Ljava/io/File;","java/io/File");
    add_virtual!(file,"listFiles(Ljava/io/FileFilter;)[Ljava/io/File;","java/io/File");
    add_virtual!(file,"getCanonicalFile()Ljava/io/File;","java/io/File");
    add_virtual!(file,"getCanonicalPath()Ljava/lang/String;","java/io/File");
    add_virtual!(file,"getParentFile()Ljava/io/File;","java/io/File");
    add_virtual!(file,"getName()Ljava/lang/String;","java/io/File");
    add_virtual!(file,"compareTo(Ljava/io/File;)I","java/io/File");
    add_virtual!(file,"setLastModified(J)Z","java/io/File");
    add_virtual!(file,"lastModified()J","java/io/File");
    add_virtual!(file,"createNewFile()Z","java/io/File");
    add_virtual!(file,"getAbsolutePath()Ljava/lang/String;","java/io/File");
    add_virtual!(file,"renameTo(Ljava/io/File;)Z","java/io/File");
    add_virtual!(file,"isDirectory()Z","java/io/File");
    add_virtual!(file,"isAbsolute()Z","java/io/File");
    add_virtual!(file,"isFile()Z","java/io/File");
    add_virtual!(file,"isHidden()Z","java/io/File");
    add_virtual!(file,"length()J","java/io/File");
    add_virtual!(file,"getParent()Ljava/lang/String;","java/io/File");
    add_virtual!(file,"toURI()Ljava/net/URI;","java/io/File");
    add_virtual!(file,"mkdirs()Z","java/io/File");
    add_virtual!(file,"exists()Z","java/io/File");
    add_virtual!(file,"delete()Z","java/io/File");  
    add_virtual!(file,"list()[Ljava/lang/String;","java/io/File");   
    add_virtual!(file,"deleteOnExit()V","java/io/File");
    exec_env.insert_class(file);
}
pub(crate) fn insert_stdio(exec_env: &mut ExecEnv) {
    if let Some(_) = exec_env.lookup_class(STDIO_INSERTION_MARKER) {
        return;
    }
    exec_env.insert_class(FatClass::new(STDIO_INSERTION_MARKER, "java/lang/Object"));
    let mut input_stream = FatClass::new("java/io/InputStream", "java/lang/Object");
    add_virtual!(input_stream,"close()V","java/io/InputStream");
    add_virtual!(input_stream,"read([B)I","java/io/InputStream");
    add_virtual!(input_stream,"read()I","java/io/InputStream");
    add_virtual!(input_stream,"read([BII)I","java/io/InputStream");
    add_virtual!(input_stream,"skip(J)J","java/io/InputStream");
    add_virtual!(input_stream,"available()I","java/io/InputStream");
    add_virtual!(input_stream,"reset()V","java/io/InputStream");
    add_virtual!(input_stream,"mark(I)V","java/io/InputStream");
    add_virtual!(input_stream,"markSupported()Z","java/io/InputStream");
    exec_env.insert_class(input_stream);
    let mut output_stream = FatClass::new("java/io/OutputStream", "java/lang/Object"); 
    add_virtual!(output_stream,"write([BII)V","java/io/OutputStream");
    add_virtual!(output_stream,"write([B)V","java/io/OutputStream");
    add_virtual!(output_stream,"write(I)V","java/io/OutputStream");
    add_virtual!(output_stream,"close()V","java/io/OutputStream");
    add_virtual!(output_stream,"flush()V","java/io/OutputStream");
     
    exec_env.insert_class(output_stream);
    let mut filter_output_stream = FatClass::new("java/io/FilterOutputStream", "java/io/OutputStream"); 
    filter_output_stream.add_field("out",FieldType::ObjectRef);
    exec_env.insert_class(filter_output_stream);
    let mut print_stream = FatClass::new("java/io/PrintStream", "java/io/FilterOutputStream"); 
    add_virtual!(print_stream,"println(Ljava/lang/String;)V","java/io/PrintStream");
    add_virtual!(print_stream,"println(Ljava/lang/Object;)V","java/io/PrintStream");
    add_virtual!(print_stream,"println(I)V","java/io/PrintStream");
    add_virtual!(print_stream,"println()V","java/io/PrintStream");
    add_virtual!(print_stream,"println(J)V","java/io/PrintStream");
    add_virtual!(print_stream,"println(D)V","java/io/PrintStream");
    add_virtual!(print_stream,"println(Z)V","java/io/PrintStream");
    add_virtual!(print_stream,"println(F)V","java/io/PrintStream");
    add_virtual!(print_stream,"printf(Ljava/lang/String;[Ljava/lang/Object;)Ljava/io/PrintStream;","java/io/PrintStream");
    add_virtual!(print_stream,"print(Ljava/lang/String;)V","java/io/PrintStream");
    add_virtual!(print_stream,"print(J)V","java/io/PrintStream");
    add_virtual!(print_stream,"print(Ljava/lang/Object;)V","java/io/PrintStream");
    let _print_stream = exec_env.insert_class(print_stream);

    let mut stdout_stream = FatClass::new("jbi/Internal/io/out", "java/io/PrintStream"); 
    const JBI_PRINT_STRING_NAME: &str = "jbi/Internal/io/out::println(Ljava/lang/String;)V";
    const JBI_PRINT_INT_NAME: &str = "jbi/Internal/io/out::println(I)V";
    stdout_stream.add_virtual("println(Ljava/lang/String;)V", JBI_PRINT_STRING_NAME);
    stdout_stream.add_virtual("println(I)V", JBI_PRINT_INT_NAME);
    let stdout_stream = exec_env.insert_class(stdout_stream).unwrap();
    let jbi_stdout_print_int = exec_env.lookup_method(JBI_PRINT_INT_NAME).unwrap();
    exec_env.replace_method_with_extern(jbi_stdout_print_int, StdoutPrintlnInt);
    let jbi_stdoutprintlnimpl_method = exec_env.lookup_method(JBI_PRINT_STRING_NAME).unwrap();
    exec_env.replace_method_with_extern(jbi_stdoutprintlnimpl_method, StdoutPrintlnString);

    let mut system = FatClass::new("java/lang/System", "java/lang/Object"); 
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
    let mut abstract_collection = FatClass::new(
        "java/util/AbstractCollection",
        "java/lang/Object",
    );
    add_virtual!(abstract_collection,"forEach(Ljava/util/function/Consumer;)V","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"addAll(Ljava/util/Collection;)Z","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"isEmpty()Z","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"stream()Ljava/util/stream/Stream;","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"clear()V","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"iterator()Ljava/util/Iterator;","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"size()I","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"add(Ljava/lang/Object;)Z","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"get(I)Ljava/lang/Object;","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"toArray()[Ljava/lang/Object;","java/util/AbstractCollection");
    add_virtual!(abstract_collection,"toArray([Ljava/lang/Object;)[Ljava/lang/Object;","java/util/AbstractCollection");
    exec_env.insert_class(abstract_collection);
    let mut abstract_list = FatClass::new(
        "java/util/AbstractList",
        "java/util/AbstractCollection",
    );
    add_virtual!(abstract_list,"listIterator()Ljava/util/ListIterator;","java/util/AbstractList");
    add_virtual!(abstract_list,"listIterator(I)Ljava/util/ListIterator;","java/util/AbstractList");
    add_virtual!(abstract_list,"remove(I)Ljava/lang/Object;","java/util/AbstractList");
    add_virtual!(abstract_list,"set(ILjava/lang/Object;)Ljava/lang/Object;","java/util/AbstractList");
    add_virtual!(abstract_list,"subList(II)Ljava/util/List;","java/util/AbstractList");
    exec_env.insert_class(abstract_list);
    let mut array_list = FatClass::new(
        "java/util/ArrayList",
        "java/util/AbstractList",
    );
    add_virtual!(array_list,"trimToSize()V","java/util/ArrayList");
    add_virtual!(array_list,"ensureCapacity(I)V","java/util/ArrayList");
    exec_env.insert_class(array_list);
    insert_path(exec_env);
    insert_file(exec_env);
    insert_proxy(exec_env);
    //todo!();
}
pub(crate) fn insert_core(exec_env: &mut ExecEnv) {
    if let Some(_) = exec_env.lookup_class(CORE_INSERTION_MARKER) {
        return;
    }
    //exec_env.insert_class(FatClass::new("java/net/URLClassLoader","java/lang/Object"));
    exec_env.insert_class(FatClass::new(CORE_INSERTION_MARKER, "java/lang/Object"));
    let mut class_loader = FatClass::new("java/lang/ClassLoader", "java/lang/Object");
    add_virtual!(class_loader,"getParent()Ljava/lang/ClassLoader;","java/lang/ClassLoader");
    add_virtual!(class_loader,"loadClass(Ljava/lang/String;)Ljava/lang/Class;","java/lang/ClassLoader");
    add_virtual!(class_loader,"getResource(Ljava/lang/String;)Ljava/net/URL;","java/lang/ClassLoader");
    add_virtual!(class_loader,"getResources(Ljava/lang/String;)Ljava/util/Enumeration;","java/lang/ClassLoader");
    add_virtual!(class_loader,"getResourceAsStream(Ljava/lang/String;)Ljava/io/InputStream;","java/lang/ClassLoader");
    exec_env.insert_class(class_loader);
    insert_record(exec_env);
    insert_string_methods(exec_env);
    insert_exceptions(exec_env);
}
pub(crate) fn insert_all(exec_env: &mut ExecEnv) {
    insert_core(exec_env);
    insert_stdio(exec_env);
}
