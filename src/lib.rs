pub(crate) type IString = Box<str>;
use crate::executor::FieldType;
use std::collections::HashMap;
mod executor;
mod importer;
mod stdlib;
mod code_container;
pub type ObjectRef = usize;
pub type ClassRef = usize;
pub type StaticRef = usize;
pub type MethodRef = usize;
pub type FieldRef = usize;
pub type VirtualMethodRef = usize;
use crate::executor::baseir::BaseIR;
use crate::executor::fatclass::FatClass;
use executor::class::Class;
use executor::ExecCtx;
use code_container::{CodeContainer,UnfinalizedMethod};
#[macro_export]
macro_rules! add_virtual{
    ($class:ident,$vmangled:literal,$class_path:literal)=>{
        $class.add_virtual(
            $vmangled,
            concat!($class_path,"::",$vmangled),
        );
    }
}
enum Method {
    BaseIR { ops: Box<[BaseIR]> },
    Invokable(Box<dyn Invokable>),
}
trait Invokable {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException>;
}
impl Method {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException> {
        match self {
            Self::Invokable(invokable) => invokable.call(ctx),
            Method::BaseIR { ops } => executor::base_runner::call(ops, ctx),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Void,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    ObjectRef(ObjectRef),
    Float(f32),
    Double(f64),
    Bool(bool),
    Char(u16), //Java chars are utf-16
}
impl Value {
    fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(a) => Some(*a),
            _ => None,
        }
    }
    fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(a) => Some(*a),
            _ => None,
        }
    }
    fn as_objref(&self) -> Option<ObjectRef> {
        match self {
            Value::ObjectRef(id) => Some(*id),
            _ => None,
        }
    }
}
#[derive(Debug)]
enum Object {
    Object {
        class_id: ClassRef,
        values: Box<[Value]>,
    },
    Array {
        //array_class_id:ClassRef,
        //element_class_id:ClassRef,
        values: Box<[Value]>,
    },
    String(IString),
}
const CLASS_TYPE_ID: ClassRef = 2;
impl Object {
    pub(crate) fn new_class(class_id: ClassRef) -> Self {
        Self::Object {
            class_id: CLASS_TYPE_ID,
            values: [Value::ObjectRef(class_id)].into(),
        }
    }
    pub(crate) fn get_array_length(&self) -> usize {
        match self {
            Self::Array { values, .. } => values.len(),
            _ => 0,
        }
    }
    pub(crate) fn set_array_at(&mut self,index:usize,value:Value){
        match self{
            Self::Object{..}=>(),
            Self::Array{values}=>values[index] = value,
            Self::String(_)=>todo!("Can't index into strings yet!"),
        }
    }
    pub(crate) fn get_array_at(&mut self,index:usize)->Value{
        match self{
            Self::Object{..}=>panic!("This is an object, so it can't be indexed into like an array!"),
            Self::Array{values}=>values[index],
            Self::String(_)=>todo!("Can't index into strings yet!"),
        }
    }
    pub fn to_string(&self) -> Option<IString> {
        match self {
            Self::Object { .. } => todo!("Can convert only strings to strings"),
            Self::String(string) => Some(string.to_owned()),
            Self::Array { .. } => todo!("Can't convert arrays to string!"),
        }
    }
    pub fn get_class(&self) -> ClassRef {
        match self {
            Self::Object { class_id, .. } => *class_id,
            Self::String(_) => 1,
            Self::Array { .. } => todo!("Can't return array class yet!"),
        }
    }
    pub fn set_field(&mut self, id: usize, value: Value) {
        //println!("seting field {id} to {value:?}");
        match self {
            Self::Object { values, .. } => values[id] = value,
            _ => (),
        }
    }
    pub fn get_field(&self, id: usize) -> Option<Value> {
        match self {
            Self::Object { values, .. } => values.get(id).cloned(),
            _ => None,
        }
    }
}
struct EnvMemory {
    objects: Vec<Object>,
    statics: Vec<Value>,
    lock: std::sync::Mutex<()>,
}
const NULL_REF: ObjectRef = 0;
impl EnvMemory {
    fn to_string(this: *mut Self, obj_ref: ObjectRef) -> Option<IString> {
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        let obj = unsafe { &(*this).objects[obj_ref] };
        let res = obj.to_string();
        drop(lock);
        res
    }
    fn get_obj_class(this: *const Self, obj: ObjectRef) -> ClassRef {
        if obj == NULL_REF {
            panic!("Null reference!");
        }
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        //unsafe{println!("objs:{:?}",(*this).objects)};
        let val = unsafe { (*this).objects[obj].get_class() };
        drop(lock);
        val
    }
    fn new_obj(this: *mut Self, new_obj: Object) -> ObjectRef {
        unsafe { (*this).objects.push(new_obj) };
        unsafe { (*this).objects.len() - 1 }
    }
    fn new_array(this: *mut Self, default_value: Value, length: usize) -> ObjectRef {
        let new_array = Object::Array {
            //element_class_id:
            values: vec![default_value; length].into(),
        };
        unsafe { (*this).objects.push(new_array) };
        unsafe { (*this).objects.len() - 1 }
    }
    fn get_static(this: *const Self, index: StaticRef) -> Value {
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        let val = unsafe { (*this).statics[index] };
        drop(lock);
        val
    }
    fn put_static(this: *mut Self, index: StaticRef,value: Value){
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        unsafe { (*this).statics[index]  = value};
        drop(lock);
    }
    pub(crate) fn get_array_length(this: *const Self, obj_ref: ObjectRef) -> usize {
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        let val = unsafe { (*this).objects[obj_ref].get_array_length() };
        drop(lock);
        val
    }
    pub(crate) fn set_array_at(this: *mut Self, arr_ref: ObjectRef,index:usize,value:Value){
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        unsafe { (*this).objects[arr_ref].set_array_at(index,value) };
        drop(lock);
    }
    pub(crate) fn get_array_at(this: *mut Self, arr_ref: ObjectRef,index:usize)->Value{
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        let value = unsafe { (*this).objects[arr_ref].get_array_at(index) };
        drop(lock);
        value
    }
    pub(crate) fn get_field(
        this: *const Self,
        obj_ref: ObjectRef,
        field_id: usize,
    ) -> Option<Value> {
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        let val = unsafe { (*this).objects[obj_ref].get_field(field_id) };
        drop(lock);
        val
    }
    fn set_field(this: *mut Self, obj_ref: ObjectRef, field_id: usize, value: Value) {
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        unsafe { (*this).objects[obj_ref].set_field(field_id, value) };
        drop(lock);
    }
    fn set_static(this: *mut Self, index: StaticRef, value: Value) {
        let lock = unsafe { (*this).lock.lock().expect("poisoned mutex!") };
        unsafe { (*this).statics[index] = value };
        drop(lock);
    }
    pub(crate) fn insert_static(&mut self, value: Value) -> usize {
        let lock = self.lock.lock().expect("poisoned mutex!");
        let index = self.statics.len();
        self.statics.push(value);
        drop(lock);
        index
    }
    fn new() -> Self {
        Self {
            objects: Vec::with_capacity(0x100),
            statics: Vec::with_capacity(0x100),
            lock: std::sync::Mutex::new(()),
        }
    }
}
pub struct ExecEnv {
    code_container: CodeContainer,
    env_mem: EnvMemory,
    //objects:Vec<Option<Object>>
}
#[test]
fn arg_counter() {
    assert_eq!(method_desc_to_argc("()I"), 0);
    assert_eq!(method_desc_to_argc("(I)I"), 1);
    assert_eq!(method_desc_to_argc("(IL)I"), 2);
    assert_eq!(method_desc_to_argc("(IJF)I"), 3);
    assert_eq!(method_desc_to_argc("(IJF)"), 3);
    assert_eq!(method_desc_to_argc("(Ljava/lang/Object;)V"), 1);
    assert_eq!(method_desc_to_argc("([[[D)V"), 1);
    //
}
fn method_desc_to_argc(desc: &str) -> u8 {
    assert_eq!(desc.chars().nth(0), Some('('));
    let mut char_beg = 0;
    let mut char_end = 0;
    for (index, character) in desc.chars().enumerate() {
        if character == '(' {
            assert_eq!(char_beg, 0);
            char_beg = index;
        } else if character == ')' {
            assert_eq!(char_end, 0);
            char_end = index;
        }
    }
    let span = &desc[(char_beg + 1)..char_end];
    let mut res = 0;
    let mut ident = false;
    for curr in span.chars() {
        if ident {
            if matches!(curr, ';') {
                ident = false;
            }
            continue;
        } else if curr == 'L' {
            ident = true;
        } else if curr == '[' {
            continue;
        }
        res += 1;
    }
    //println!("span:{span},res{res}");
    res as u8
}
fn mangle_method_name(class: &str, method: &str, desc: &str) -> IString {
    format!("{class}::{method}{desc}").into_boxed_str()
}
fn mangle_method_name_partial(method: &str, desc: &str) -> IString {
    format!("{method}{desc}").into_boxed_str()
}
impl ExecEnv {
    pub(crate) fn lookup_virtual(&self,class_id:ClassRef,method_name:&str)->Option<VirtualMethodRef>{
        self.code_container.lookup_virtual(class_id,method_name)
    }
    pub(crate) fn get_static(&self,class_id:ClassRef,static_name:&str)->Option<StaticRef>{
        self.code_container.get_static(class_id,static_name)
    }
    pub(crate) fn get_field(&self,class_id:ClassRef,field_name:&str)->Option<(FieldRef, &FieldType)>{
        self.code_container.get_field(class_id,field_name)
    }
    /// Searches for method named main, which returns void and accepts an array of strings. this function may also be provided with
    /// an optional hint, eg. name of the class the desired method resides in. It will either return first method it finds,
    /// or None, if no method matches all requirements.
    /// (**WARNING!** If two or more methods fullfill all requirements, it may return any method that fullfills all requirements. 
    /// There is **no guarantee** that this will be consistent between **runs of the same program**).
    /// Class paths in optional hint should be internal java paths, eg. `java/lang/Object` instead of `java.lang.Object`.
    pub fn try_find_main(&self,hint:Option<&str>)->Option<MethodRef>{
        for (name,id) in self.code_container.method_names(){
            if name.contains("::main([Ljava/lang/String;)V"){
                if let Some(hint) = hint{
                    if name.contains(hint){
                        return Some(*id);
                    }
                }
                else{
                    return Some(*id);
                }
                //println!("{name} with id {id} is likely the main method!");
                
            }
        }
        None
    }
    fn get_class_virtual_methods(&self, class_id: ClassRef) -> Option<Box<[usize]>> {
        Some(
            self.code_container
                .get_class(class_id)?
                .virtual_methods()
                .into(),
        )
    }
    fn get_class_virtual_map(&self, class_id: ClassRef) -> Option<HashMap<IString, usize>> {
        Some(
            self.code_container
                .get_class(class_id)?
                .virtual_map()
                .clone(),
        )
    }
    fn get_class_statics_map(&self, class_id: ClassRef) -> Option<HashMap<IString, usize>> {
        Some(
            self.code_container
                .get_class(class_id)?
                .statics_map()
                .clone(),
        )
    }
    fn get_class_field_types(
        &self,
        class_id: ClassRef,
    ) -> Option<Box<[crate::executor::FieldType]>> {
        Some(
            self.code_container
                .get_class(class_id)?
                .field_types()
                .into(),
        )
    }
    fn get_class_field_map(&self, class_id: ClassRef) -> Option<HashMap<IString, usize>> {
        Some(
            self.code_container
                .get_class(class_id)?
                .field_map()
                .clone(),
        )
    }
    fn dep_status(&self) {
        let uclasses = self.code_container.unfinalized_class_count();
        let fclasses = self.code_container.classes().len();
        println!("Class finalization status {}%, Finalized: {}, Unfinalized: {}",
            (fclasses as f64)/((uclasses + fclasses) as f64)*100.0,
            fclasses,
            uclasses,
        );
        let umethods = self.code_container.unfinalized_methods.len();
        let methods = self.code_container.methods().len();
        println!("Method finalization status {}%, Total: {}, Unfinalized: {}",
            (1.0 - (umethods as f64)/(methods as f64))*100.0,
            methods,
            umethods,
        );
    }
    fn const_string(&mut self, string: &str) -> ObjectRef {
        *self
            .code_container
            .static_strings()
            .entry(string.into())
            .or_insert_with(|| {
                let new_obj = Object::String(string.into());
                let obj_ref: ObjectRef = EnvMemory::new_obj(&mut self.env_mem as *mut _, new_obj);
                //Prevent GC from cleaning it up.
                self.env_mem.insert_static(Value::ObjectRef(obj_ref));
                obj_ref
            })
    }
    fn const_class(&mut self, class_id: ClassRef) -> ObjectRef {
        *self
            .code_container
            .const_classes()
            .entry(class_id)
            .or_insert_with(|| {
                let new_obj = Object::new_class(class_id);
                let obj_ref: ObjectRef = EnvMemory::new_obj(&mut self.env_mem as *mut _, new_obj);
                //Prevent GC from cleaning it up.
                self.env_mem.insert_static(Value::ObjectRef(obj_ref));
                obj_ref
            })
    }
    /// Creates a new [`ExecEnv`] with a small, essential part of stdlib already present.
    /// Call [`Self::insert_stdlib`] to add the rest of suported parts of the standard library.
    pub fn new() -> Self {
        use crate::executor::FieldType;
        let env_mem = EnvMemory::new();
        let code_container = CodeContainer::new();
        //let objects = vec!
        let mut res = Self {
            code_container,
            env_mem,
        };
        //let obj_class = res.lookup_class("java/lang/Object").unwrap();
        let mut object_class = FatClass::new("java/lang/Object", "java/lang/Object");
        add_virtual!(object_class,"getClass()Ljava/lang/Class;","java/lang/Object");
        add_virtual!(object_class,"equals(Ljava/lang/Object;)Z","java/lang/Object");
        add_virtual!(object_class,"hashCode()I","java/lang/Object");
        add_virtual!(object_class,"toString()Ljava/lang/String;","java/lang/Object");
        add_virtual!(object_class,"wait()V","java/lang/Object");
        add_virtual!(object_class,"wait(JI)V","java/lang/Object");
        add_virtual!(object_class,"notifyAll()V","java/lang/Object");
        add_virtual!(object_class,"clone()Ljava/lang/Object;","java/lang/Object");
        let final_object_class = crate::executor::class::finalize(&object_class, &mut res).unwrap();
        let obj_class = res
            .code_container
            .set_or_replace_class("java/lang/Object", final_object_class);
        let null_obj = res.new_obj(obj_class);
        res.env_mem.insert_static(Value::ObjectRef(null_obj));
        let obj_init = res
            .code_container
            .lookup_or_insert_method("java/lang/Object::<init>()V");
        res.replace_method_with_extern(obj_init, || {});
        let mut string = FatClass::new("java/lang/String", "java/lang/Object");
        add_virtual!(string,"split(Ljava/lang/String;)[Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"startsWith(Ljava/lang/String;)Z","java/lang/String");
        add_virtual!(string,"length()I","java/lang/String");
        add_virtual!(string,"regionMatches(ILjava/lang/String;II)Z","java/lang/String");
        add_virtual!(string,"charAt(I)C","java/lang/String");
        add_virtual!(string,"isEmpty()Z","java/lang/String");
        add_virtual!(string,"trim()Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"regionMatches(ZILjava/lang/String;II)Z","java/lang/String");
        add_virtual!(string,"indexOf(II)I","java/lang/String");
        add_virtual!(string,"indexOf(I)I","java/lang/String");
        add_virtual!(string,"toLowerCase()Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"substring(I)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"substring(II)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"replace(CC)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"getBytes(Ljava/nio/charset/Charset;)[B","java/lang/String");
        add_virtual!(string,"getBytes()[B","java/lang/String");
        add_virtual!(string,"getBytes(Ljava/lang/String;)[B","java/lang/String");
        add_virtual!(string,"toCharArray()[C","java/lang/String");
        add_virtual!(string,"equalsIgnoreCase(Ljava/lang/String;)Z","java/lang/String");
        add_virtual!(string,"endsWith(Ljava/lang/String;)Z","java/lang/String");
        add_virtual!(string,"getChars(II[CI)V","java/lang/String");
        add_virtual!(string,"lastIndexOf(I)I","java/lang/String");
        add_virtual!(string,"contains(Ljava/lang/CharSequence;)Z","java/lang/String");
        add_virtual!(string,"indexOf(Ljava/lang/String;)I","java/lang/String");
        add_virtual!(string,"lastIndexOf(Ljava/lang/String;)I","java/lang/String");
        add_virtual!(string,"split(Ljava/lang/String;I)[Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"indexOf(Ljava/lang/String;I)I","java/lang/String");
        add_virtual!(string,"lastIndexOf(II)I","java/lang/String");
        add_virtual!(string,"lastIndexOf(Ljava/lang/String;I)I","java/lang/String");
        add_virtual!(string,"replaceAll(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"replaceFirst(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"forEach(Ljava/util/function/Consumer;)V","java/lang/String");
        add_virtual!(string,"compareTo(Ljava/lang/String;)I","java/lang/String");
        add_virtual!(string,"compareToIgnoreCase(Ljava/lang/String;)I","java/lang/String");
        add_virtual!(string,"toUpperCase()Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"concat(Ljava/lang/String;)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"matches(Ljava/lang/String;)Z","java/lang/String");
        add_virtual!(string,"codePoints()Ljava/util/stream/IntStream;","java/lang/String");
        add_virtual!(string,"chars()Ljava/util/stream/IntStream;","java/lang/String");
        add_virtual!(string,"startsWith(Ljava/lang/String;I)Z","java/lang/String");
        add_virtual!(string,"codePointAt(I)I","java/lang/String");
        add_virtual!(string,"codePointCount(II)I","java/lang/String");
        add_virtual!(string,"toLowerCase(Ljava/util/Locale;)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"toUpperCase(Ljava/util/Locale;)Ljava/lang/String;","java/lang/String");
        add_virtual!(string,"replace(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;","java/lang/String");
        string.add_static("CASE_INSENSITIVE_ORDER",FieldType::Bool);
        //
        res.insert_class(string);
        let mut class_class = FatClass::new("java/lang/Class", "java/lang/Object");
        add_virtual!(class_class,"getClassLoader()Ljava/lang/ClassLoader;","java/lang/Class");
        add_virtual!(class_class,"getResourceAsStream(Ljava/lang/String;)Ljava/io/InputStream;","java/lang/Class");
        add_virtual!(class_class,"desiredAssertionStatus()Z","java/lang/Class");
        add_virtual!(class_class,"getTypeParameters()[Ljava/lang/reflect/TypeVariable;","java/lang/Class");
        add_virtual!(class_class,"getDeclaredField(Ljava/lang/String;)Ljava/lang/reflect/Field;","java/lang/Class");
        add_virtual!(class_class,"getTypeName()Ljava/lang/String;","java/lang/Class");
        add_virtual!(class_class,"getName()Ljava/lang/String;","java/lang/Class");
        add_virtual!(class_class,"getSimpleName()Ljava/lang/String;","java/lang/Class");
        add_virtual!(class_class,"getSuperclass()Ljava/lang/Class;","java/lang/Class");
        add_virtual!(class_class,"getDeclaredMethod(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;","java/lang/Class");
        add_virtual!(class_class,"isInstance(Ljava/lang/Object;)Z","java/lang/Class");
        add_virtual!(class_class,"getConstructor([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;","java/lang/Class");
        add_virtual!(class_class,"getMethod(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;","java/lang/Class");
        add_virtual!(class_class,"getPackage()Ljava/lang/Package;","java/lang/Class");
        add_virtual!(class_class,"newInstance()Ljava/lang/Object;","java/lang/Class");
        add_virtual!(class_class,"getResource(Ljava/lang/String;)Ljava/net/URL;","java/lang/Class");
        add_virtual!(class_class,"getComponentType()Ljava/lang/Class;","java/lang/Class");
        add_virtual!(class_class,"getDeclaredMethods()[Ljava/lang/reflect/Method;","java/lang/Class");
        add_virtual!(class_class,"isArray()Z","java/lang/Class");
        add_virtual!(class_class,"isAnonymousClass()Z","java/lang/Class");
        add_virtual!(class_class,"isPrimitive()Z","java/lang/Class");
        add_virtual!(class_class,"isEnum()Z","java/lang/Class");
        add_virtual!(class_class,"isAnnotation()Z","java/lang/Class");
        add_virtual!(class_class,"getInterfaces()[Ljava/lang/Class;","java/lang/Class");
        add_virtual!(class_class,"getEnclosingClass()Ljava/lang/Class;","java/lang/Class");
        add_virtual!(class_class,"getCanonicalName()Ljava/lang/String;","java/lang/Class");
        add_virtual!(class_class,"getEnumConstants()[Ljava/lang/Object;","java/lang/Class");
        add_virtual!(class_class,"isAssignableFrom(Ljava/lang/Class;)Z","java/lang/Class");
        add_virtual!(class_class,"getDeclaredFields()[Ljava/lang/reflect/Field;","java/lang/Class");
        add_virtual!(class_class,"isInterface()Z","java/lang/Class");
        add_virtual!(class_class,"cast(Ljava/lang/Object;)Ljava/lang/Object;","java/lang/Class");
        add_virtual!(class_class,"getConstructors()[Ljava/lang/reflect/Constructor;","java/lang/Class");
        add_virtual!(class_class,"getModifiers()I","java/lang/Class");
        add_virtual!(class_class,"getMethods()[Ljava/lang/reflect/Method;","java/lang/Class");
        add_virtual!(class_class,"getGenericInterfaces()[Ljava/lang/reflect/Type;","java/lang/Class");
        add_virtual!(class_class,"asSubclass(Ljava/lang/Class;)Ljava/lang/Class;","java/lang/Class");
        add_virtual!(class_class,"getDeclaredConstructor([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;","java/lang/Class");
        add_virtual!(class_class,"getProtectionDomain()Ljava/security/ProtectionDomain;","java/lang/Class");
        add_virtual!(class_class,"getGenericSuperclass()Ljava/lang/reflect/Type;","java/lang/Class");
        add_virtual!(class_class,"isMemberClass()Z","java/lang/Class");
        add_virtual!(class_class,"isLocalClass()Z","java/lang/Class");
        add_virtual!(class_class,"getSigners()[Ljava/lang/Object;","java/lang/Class");
        add_virtual!(class_class,"arrayType()Ljava/lang/Class;","java/lang/Class");
        res.insert_class(class_class);
        let mut number_class =FatClass::new("java/lang/Number", "java/lang/Object");
        add_virtual!(number_class,"intValue()I","java/lang/Number");
        add_virtual!(number_class,"shortValue()S","java/lang/Number");
        add_virtual!(number_class,"longValue()J","java/lang/Number");
        add_virtual!(number_class,"byteValue()B","java/lang/Number");
        add_virtual!(number_class,"doubleValue()D","java/lang/Number");
        add_virtual!(number_class,"floatValue()F","java/lang/Number");
        res.insert_class(number_class);
        let mut intiger_class = FatClass::new("java/lang/Integer", "java/lang/Number");
        add_virtual!(intiger_class,"intValue()I","java/lang/Integer");
        add_virtual!(intiger_class,"compareTo(Ljava/lang/Integer;)I","java/lang/Integer");
        //add_virtual!(intiger_class,"intValue()I","java/lang/Integer");
        intiger_class.add_static("TYPE",FieldType::ObjectRef);
        res.insert_class(intiger_class);
        let mut enum_class = FatClass::new("java/lang/Enum", "java/lang/Object");
        add_virtual!(enum_class,"ordinal()I","java/lang/Enum");
        add_virtual!(enum_class,"compareTo(Ljava/lang/Enum;)I","java/lang/Enum");
        add_virtual!(enum_class,"name()Ljava/lang/String;","java/lang/Enum");
        add_virtual!(enum_class,"getDeclaringClass()Ljava/lang/Class;","java/lang/Enum");
        res.insert_class(enum_class);
        res
    }
    pub(crate) fn load_method(
        &mut self,
        method: &crate::importer::Method,
        class: &crate::importer::ImportedJavaClass,
    ) {
        let bytecode = if let Some(bytecode) = method.bytecode() {
            bytecode
        } else {
            self.code_container.methods_mut().push(None);
            return;
        };
        let fat = crate::executor::fatops::expand_ops(bytecode, &class);
        let _max_locals = method.max_locals().unwrap();
        let (name, descriptor) = (method.name_index(), method.descriptor_index());
        let method_class = class.lookup_class(class.this_class()).unwrap();
        let name = class.lookup_utf8(name).unwrap();
        let descriptor = class.lookup_utf8(descriptor).unwrap();
        let mangled = mangle_method_name(method_class, name, descriptor);
        let _method_id = self.code_container.lookup_or_insert_method(&mangled);
        //let argc = method_desc_to_argc(&descriptor);
        let af = method.access_flags();
        let _is_static = af.is_static();
        //println!("mangled:{mangled}");
        let method_id = self.code_container.lookup_or_insert_method(&mangled);
        self.insert_method(&fat, method_id);
    }
    fn insert_method(&mut self, ops: &[crate::executor::fatops::FatOp], method_id:MethodRef) {
        match crate::executor::baseir::into_base(&ops, self) {
            Ok(baseir) => {
                //println!("{mangled:?} -> {method_id}");
                self.code_container.methods_mut()[method_id] = Some(Method::BaseIR { ops: baseir })
            }
            Err(err) => self
                .code_container
                .unfinalized_methods
                .entry(err.dependency().into())
                .or_insert(Vec::new())
                .push(UnfinalizedMethod {
                    ops: ops.into(),
                    method_id,
                }),
        }
    }
    pub(crate) fn insert_dependant_class(&mut self,dependency_name:&str){
        if let Some(dependants) = self.code_container.unfinalized_classes.remove(dependency_name){
            for dependant in dependants{
                self.insert_class(dependant);
            }
        }
    }
    pub(crate) fn insert_dependant_method(&mut self,dependency_name:&str){
        if let Some(dependants) = self.code_container.unfinalized_methods.remove(dependency_name){
            for dependant in dependants{
                self.insert_method(&dependant.ops,dependant.method_id);
            }
        }
    }
    pub(crate) fn insert_class(
        &mut self,
        base_class: crate::executor::fatclass::FatClass,
    ) -> Option<ClassRef> {
        match crate::executor::class::finalize(&base_class, self) {
            Ok(final_class) => {
                let cref = self.code_container.set_or_replace_class(base_class.class_name(), final_class);
                self.insert_dependant_class(base_class.class_name());
                self.insert_dependant_method(base_class.class_name());
                Some(cref)
            },
            Err(err) => {
                self.code_container.add_unfinalized_class(err.dependency(),base_class);
                None
            }
        }
    }
    pub(crate) fn load_class(&mut self, class: crate::importer::ImportedJavaClass) {
        let base_class = crate::executor::fatclass::expand_class(&class);
        self.insert_class(base_class);
        for method in class.methods() {
            self.load_method(method, &class);
        }
    }
    pub(crate) fn lookup_method(&self, method_name: &str) -> Option<MethodRef> {
        self.code_container.method_names().get(method_name).copied()
    }
    pub(crate) fn replace_method_with_extern<T: Invokable + 'static>(
        &mut self,
        methodref: MethodRef,
        extern_fn: T,
    ) {
        self.code_container.methods_mut()[methodref] = Some(Method::Invokable(Box::new(extern_fn)));
    }
    pub(crate) fn lookup_class(&self, class_name: &str) -> Option<usize> {
        self.code_container.lookup_class(class_name)
    }
    pub(crate) fn new_obj(&mut self, class: ClassRef) -> ObjectRef {
        let new_obj = self.code_container.new_obj(class);
        EnvMemory::new_obj(&mut self.env_mem as *mut _, new_obj)
    }
    pub(crate) fn get_static_id(&mut self, class: ClassRef, name: &str) -> Option<StaticRef> {
        self.code_container.classes()[class].get_static(name)
    }
    pub(crate) fn set_static(&mut self, index: StaticRef, value: Value) {
        EnvMemory::set_static(&mut self.env_mem as *mut _, index, value)
    }
    
    pub fn call_method(
        &mut self,
        method_id: MethodRef,
        args: &[Value],
    ) -> Result<Value, ExecException> {
        let args: Vec<_> = args.into();
        //args.reverse();
        let e_ctx = executor::ExecCtx::new(&mut self.env_mem, &self.code_container, &args, 6);
        //todo!();
        let method = self.code_container.methods().get(method_id);
        let method_ref = method
            .ok_or(ExecException::MethodNotFound)?
            .as_ref();
        if let Some(method) = method_ref{
            method.call(e_ctx)
        }
        else{
            self.code_container.diagnose_method(method_id);
            Err(ExecException::MethodNotFound)
        }
     
    }
    pub fn insert_stdlib(&mut self) {
        stdlib::insert_all(self);
    }
    pub fn import_jar<R:std::io::Read + std::io::Seek>(&mut self,jar:&mut R)->Result<(),crate::importer::BytecodeImportError>{
        let classes =  importer::load_jar(jar)?;
        for class in classes{  
            self.load_class(class);
        }
        Ok(())
    }
    pub fn import_class<R:std::io::Read>(&mut self,jar:&mut R)->Result<(),crate::importer::BytecodeImportError>{
        let class = importer::load_class(jar)?;
        self.load_class(class);
        Ok(())
    }
}
//trait SimpleARG{};
impl<T: Fn()> Invokable for T {
    fn call(&self, _: ExecCtx) -> Result<Value, ExecException> {
        Ok(Value::Void)
    }
}
#[derive(Debug)]
pub enum ExecException {
    MethodNotFound,
}
#[test]
fn exec_identity() {
    let mut file = std::fs::File::open("test/Identity.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let identity = exec_env
        .lookup_method(&mangle_method_name("Identity", "Identity", "(I)I"))
        .unwrap();
    for a in 0..1000 {
        assert_eq!(
            exec_env.call_method(identity, &[Value::Int(a)]).unwrap(),
            Value::Int(a)
        );
    }
}

#[test]
fn basic_arthm() {
    let mut file = std::fs::File::open("test/BasicArthm.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let add = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Add", "(II)I"))
        .unwrap();
    let sub = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Sub", "(II)I"))
        .unwrap();
    let mul = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Mul", "(II)I"))
        .unwrap();
    let div = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Div", "(II)I"))
        .unwrap();
    let rem = exec_env
        .lookup_method(&mangle_method_name("BasicArthm", "Mod", "(II)I"))
        .unwrap();
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env
                    .call_method(add, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a + b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env
                    .call_method(sub, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a - b)
            );
        }
    }
    for a in 0..100 {
        for b in 0..100 {
            assert_eq!(
                exec_env
                    .call_method(mul, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a * b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env
                    .call_method(div, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a / b)
            );
        }
    }
    for a in 1..100 {
        for b in 1..100 {
            assert_eq!(
                exec_env
                    .call_method(rem, &[Value::Int(a), Value::Int(b)])
                    .unwrap(),
                Value::Int(a % b)
            );
        }
    }
}
struct AddFiveInvokable;
struct SqrtInvokable;
impl Invokable for AddFiveInvokable {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException> {
        Ok(Value::Int(ctx.get_local(0).unwrap().as_int().unwrap() + 5))
    }
}
impl Invokable for SqrtInvokable {
    fn call(&self, ctx: ExecCtx) -> Result<Value, ExecException> {
        Ok(Value::Float(
            ctx.get_local(0).unwrap().as_float().unwrap().sqrt(),
        ))
    }
}
#[test]
fn exec_call() {
    let mut file = std::fs::File::open("test/Calls.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let sqr_mag = exec_env
        .lookup_method(&mangle_method_name("Calls", "SqrMag", "(III)I"))
        .unwrap();
    let first = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnFirst", "(IIIII)I"))
        .unwrap();
    let second = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnSecond", "(IIIII)I"))
        .unwrap();
    let last = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnLast", "(IIIII)I"))
        .unwrap();
    let first_bck = exec_env
        .lookup_method(&mangle_method_name("Calls", "ReturnFirst", "(IIIII)I"))
        .unwrap();
    assert_eq!(
        exec_env
            .call_method(
                first_bck,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(1)
    );
    assert_eq!(
        exec_env
            .call_method(
                first,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(1)
    );
    assert_eq!(
        exec_env
            .call_method(
                second,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(2)
    );
    assert_eq!(
        exec_env
            .call_method(
                last,
                &[
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                    Value::Int(4),
                    Value::Int(5)
                ]
            )
            .unwrap(),
        Value::Int(5)
    );
    for a in 0..1000 {
        exec_env
            .call_method(sqr_mag, &[Value::Int(a), Value::Int(7), Value::Int(8)])
            .unwrap();
    }
    let extern_call = exec_env
        .lookup_method(&mangle_method_name("Calls", "ExternCallTest", "(I)I"))
        .unwrap();
    for a in -1000..1000 {
        assert_eq!(
            exec_env.call_method(extern_call, &[Value::Int(a)]).unwrap(),
            Value::Int(0)
        );
    }
    exec_env.replace_method_with_extern(extern_call,AddFiveInvokable);
    for a in -1000..1000 {
        assert_eq!(
            exec_env.call_method(extern_call, &[Value::Int(a)]).unwrap(),
            Value::Int(a + 5)
        );
    }
}

#[test]
fn exec_hw() {
    let mut file = std::fs::File::open("test/HelloWorld.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.insert_stdlib();
    exec_env.load_class(class);
    let hw = exec_env
        .lookup_method(&mangle_method_name(
            "HelloWorld",
            "main",
            "([Ljava/lang/String;)V",
        ))
        .unwrap();
    exec_env.call_method(hw, &[]).unwrap();
    //panic!();
}
#[test]
fn fields() {
    let mut file = std::fs::File::open("test/Fields.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);

    //let hw = exec_env.lookup_method(&mangle_method_name("HelloWorld","main","([Ljava/lang/String;)V")).unwrap();
    //exec_env.call_method(hw,&[]).unwrap();
}
#[test]
fn gravity() {
    let mut file = std::fs::File::open("test/Gravity.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(class);
    let tick = exec_env
        .lookup_method(&mangle_method_name("Gravity", "Tick", "()V"))
        .unwrap();
    let set = exec_env
        .lookup_method(&mangle_method_name("Gravity", "SetPos", "(FF)V"))
        .unwrap();
    let set_vel = exec_env
        .lookup_method(&mangle_method_name("Gravity", "SetVel", "(FF)V"))
        .unwrap();
    let getx = exec_env
        .lookup_method(&mangle_method_name("Gravity", "GetX", "()F"))
        .unwrap();
    let gety = exec_env
        .lookup_method(&mangle_method_name("Gravity", "GetY", "()F"))
        .unwrap();
    let class = exec_env.lookup_class("Gravity").unwrap();
    let sqrt_extern_call = exec_env
        .lookup_method(&mangle_method_name("Gravity", "Sqrt", "(F)F"))
        .unwrap();
    exec_env.replace_method_with_extern(sqrt_extern_call,SqrtInvokable);
    let obj = exec_env.new_obj(class);
    exec_env
        .call_method(
            set,
            &[
                Value::ObjectRef(obj),
                Value::Float(0.43),
                Value::Float(203.23),
            ],
        )
        .unwrap();
    exec_env
        .call_method(
            set_vel,
            &[
                Value::ObjectRef(obj),
                Value::Float(0.06125),
                Value::Float(0.0),
            ],
        )
        .unwrap();
    for i in 0..10_000 {
        exec_env
            .call_method(tick, &[Value::ObjectRef(obj)])
            .unwrap();
    }
    //panic!();
}
#[test]
fn extends() {
    let mut file = std::fs::File::open("test/SuperClass.class").unwrap();
    let super_class = crate::importer::load_class(&mut file).unwrap();
    let mut file = std::fs::File::open("test/Extends.class").unwrap();
    let class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(super_class);
    exec_env.load_class(class);
}
#[test]
fn nbody() {
    let mut file = std::fs::File::open("test/nbody/Vector3.class").unwrap();
    let vec3_class = crate::importer::load_class(&mut file).unwrap();
    let mut file = std::fs::File::open("test/nbody/Planet.class").unwrap();
    let planet_class = crate::importer::load_class(&mut file).unwrap();
    let mut file = std::fs::File::open("test/nbody/NBody.class").unwrap();
    let nbody_class = crate::importer::load_class(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    exec_env.load_class(vec3_class);
    exec_env.load_class(planet_class);
    exec_env.load_class(nbody_class);
    let new_nbody = exec_env
        .lookup_method(&mangle_method_name("NBody", "NewNBody", "(I)LNBody;"))
        .unwrap();
    let tick = exec_env
        .lookup_method(&mangle_method_name("NBody", "Tick", "()V"))
        .unwrap();
    let nbody = exec_env.call_method(new_nbody, &[Value::Int(10)]).unwrap();
    exec_env.call_method(tick, &[nbody]).unwrap();
}

#[test]
fn load_jar() {
    let mut file = std::fs::File::open("test/jopt.jar").unwrap();
    let classes = importer::load_jar(&mut file).unwrap();
    let mut exec_env = ExecEnv::new();
    stdlib::insert_all(&mut exec_env);
    for class in classes{
        let name = class.name().to_owned();
        //println!("joptclass:{name}");
        exec_env.load_class(class);
        //exec_env.code_container.class_alias(&name,"joptsimple/");
    }
    //panic!();
    let id = exec_env.lookup_class("joptsimple/OptionSet").unwrap();
    let mut file = std::fs::File::open("test/server.jar").unwrap();
    let classes = importer::load_jar(&mut file).unwrap();
    for class in classes{
        exec_env.load_class(class);
    }
    exec_env.dep_status();
    let main = exec_env.try_find_main(Some("server")).expect("Can't find main!");
    let argc_ref = 0;
    exec_env.call_method(main,&[Value::ObjectRef(argc_ref)]).unwrap();
    panic!();
}
#[test]
fn load_interface() {
    let mut exec_env = ExecEnv::new();
    stdlib::insert_all(&mut exec_env);
    let mut file = std::fs::File::open("test/interface/Animal.class").unwrap();
    let class = importer::load_class(&mut file).unwrap();
    exec_env.load_class(class);
    let mut file = std::fs::File::open("test/interface/Dog.class").unwrap();
    let class = importer::load_class(&mut file).unwrap();
    exec_env.load_class(class);
    let mut file = std::fs::File::open("test/interface/Tester.class").unwrap();
    let class = importer::load_class(&mut file).unwrap();
    exec_env.load_class(class);
    //panic!();
}

