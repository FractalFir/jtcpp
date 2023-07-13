use crate::{Class, IString};
use std::collections::HashSet;
use std::io::Write;
pub mod method;
pub(crate) use method::create_method_impl;
pub(self) struct IncludeBuilder {
    header: String,
    includes: HashSet<IString>,
}
fn create_namespace_def(cpp_class_name:&str)->IString{
    let mut iter:Vec<_> = cpp_class_name.split("::").collect();
    iter.reverse();
    let mut iter = iter.iter();
    let mut res = String::new();
    if let Some(class_name) = iter.next(){
        res = format!("struct {class_name};");
    }
    for namespace in iter{
        res = format!("namespace {namespace}{{{res}}};");
    }
    res.into()
}
#[test]
fn class_name_to_namespace_def(){
    assert_eq!(&*create_namespace_def("java::lang::Object"),"namespace java{namespace lang{struct Object;};};");
    assert_eq!(&*create_namespace_def("Vector3"),"struct Vector3;");
}
impl IncludeBuilder {
    fn new(this_file: &str) -> Self {
        Self {
            header: String::new(),
            includes: [this_file.into()].into(),
        }
    }
    fn add_include(&mut self, include: &str) {
        if self.includes.get(include).is_none() {
            self.header
                .push_str(&format!("#include \"{include}.hpp\"\n"));
            self.includes.insert(include.into());
        }
    }
    pub(crate) fn get_code(&self) -> &str {
        &self.header
    }
}
fn push_method_sig(target: &mut String, method_name: &str, method: &crate::Method) {
    target.push_str(&format!(
        "{ret} {method_name}(",
        ret = method.ret_val().c_type()
    ));
    let mut margs = method.args().iter();
    //println!("\n\t{name}::{method_name}->{margs:?}",name = self.name);
    if let Some(arg) = margs.next() {
        target.push_str(&arg.c_type())
    };
    for marg in margs {
        target.push(',');
        target.push_str(&marg.c_type());
    }
    target.push(')');
}
pub(crate) fn create_header<W: Write>(out: &mut W, class: &Class) -> std::io::Result<()> {
    let mut includes = IncludeBuilder::new(&*class.path());
    includes.add_include(&*class.parrent_path());
    let mut class_methods = String::new();
    for (method_name, method) in class.static_methods() {
        class_methods.push_str("\tstatic ");
        push_method_sig(&mut class_methods, method_name, method);
        // Dependencies
        for arg in method.args() {
            if let Some(dep) = arg.dependency() {
                includes.add_include(&dep);
            }
        }
        class_methods.push_str(";\n");
        if let Some(dep) = method.ret_val().dependency() {
            includes.add_include(&dep);
        }
    }
    for (method_name, method) in class.virtual_methods() {
        class_methods.push_str("\tvirtual ");
        push_method_sig(&mut class_methods, method_name, method);
        // Dependencies
        for arg in method.args() {
            if let Some(dep) = arg.dependency() {
                includes.add_include(&dep);
            }
        }
        class_methods.push_str(";\n");
        if let Some(dep) = method.ret_val().dependency() {
            includes.add_include(&dep);
        }
    }
    let mut class_fields = String::new();
    for (field_name, field_type) in class.static_fields() {
        class_fields.push_str(&format!(
            "\tstatic {ctype} {field_name};\n",
            ctype = field_type.c_type()
        ));
        if let Some(dep) = field_type.dependency() {
            includes.add_include(&dep);
        }
    }
    for (field_name, field_type) in class.fields() {
        class_fields.push_str(&format!(
            "\t{ctype} {field_name};\n",
            ctype = field_type.c_type()
        ));
        if let Some(dep) = field_type.dependency() {
            includes.add_include(&dep);
        }
    }
    if class.cpp_name().contains("::") {
        write!(out,"{}\n",create_namespace_def(class.cpp_name()))?;
    }
    write!(
        out,
        "#pragma once\n{includes}\nstruct {name}: {super_name}{{\n{class_fields}{class_methods}}};",
        includes = includes.get_code(),
        name = class.cpp_name(),
        super_name = class.parrent_cpp_name()
    )
}
