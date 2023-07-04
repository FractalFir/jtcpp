use crate::{IString,VariableType,Method,class_path_to_class_mangled,ERR_SUPER_INVALID,ERR_THIS_INVALID,field_descriptor_to_ftype};
use std::io::Write;
pub(crate) struct Class {
    name: IString,
    parrent: IString,
    fields: Vec<(IString, VariableType)>,
    static_fields: Vec<(IString, VariableType)>,
    static_methods: Vec<(IString, Method)>,
    virtual_methods: Vec<(IString, Method)>,
}
impl Class {
    pub(crate) fn name(&self)->&str{
        &self.name
    }
    pub(crate) fn static_methods(&self)->&[(IString,Method)]{
        &self.static_methods
    }
    pub(crate) fn virtual_methods(&self)->&[(IString,Method)]{
        &self.virtual_methods
    }
    pub(crate) fn fields(&self)->&[(IString,VariableType)]{
        &self.fields
    }
    pub(crate) fn statics_fields(&self)->&[(IString,VariableType)]{
        &self.static_fields
    }
    pub(crate) fn from_java_class(java_class: &crate::importer::ImportedJavaClass) -> Self {
        let name = match java_class.lookup_class(java_class.this_class()) {
            Some(name) => name,
            None => {
                eprintln!(
                    "\nMalformed class file! `this_class` field did not refer to a valid class!"
                );
                std::process::exit(ERR_THIS_INVALID);
            }
        };
        let class_name = class_path_to_class_mangled(name);
        let parrent = match java_class.lookup_class(java_class.super_class()) {
            Some(parrent) => parrent,
            None => {
                eprintln!(
                    "\nMalformed class file! `super_class` field did not refer to a valid class!"
                );
                std::process::exit(ERR_SUPER_INVALID);
            }
        };
        let parrent = class_path_to_class_mangled(parrent);
        let mut fields: Vec<(IString, VariableType)> =
            Vec::with_capacity(java_class.fields().len());
        let mut static_fields: Vec<(IString, VariableType)> =
            Vec::with_capacity(java_class.fields().len());
        for field in java_class.fields() {
            let (name_index, descriptor_index) = (field.name_index, field.descriptor_index);
            let name = java_class.lookup_utf8(name_index).unwrap();
            let ftype = field_descriptor_to_ftype(descriptor_index, java_class);
            if field.flags.is_static() {
                static_fields.push((name.into(), ftype));
            } else {
                fields.push((name.into(), ftype));
            }
        }
        let mut static_methods: Vec<(IString, Method)> =
            Vec::with_capacity(java_class.methods().len());
        let mut virtual_methods: Vec<(IString, Method)> =
            Vec::with_capacity(java_class.methods().len());
        for method in java_class.methods() {
            if method.is_virtual(java_class) || method.name(java_class).contains("<init>"){
                let mangled_name = method.virtual_name(java_class);
                let mut method = Method::from_raw_method(&method, &mangled_name, &java_class);
                virtual_methods.push((mangled_name, method));
            } else {
                let mangled_name = method.mangled_name(java_class);
                let method = Method::from_raw_method(&method, &mangled_name, &java_class);
                static_methods.push((mangled_name, method));
            }
        }
        Class {
            name: class_name,
            parrent,
            fields,
            static_fields,
            static_methods,
            virtual_methods,
        }
        //todo!("name:{name} parrent:{parrent} fields:{fields:?} static_fields:{static_fields:?}");
    }
    pub(crate) fn write_header<W: Write>(&self, hout: &mut W) -> std::io::Result<()> {
        let super_name = &self.parrent;
        let class_name = self.name();
        let mut includes = format!("#include <memory>\n#include \"{super_name}.hpp\"\n");
        let mut class_fields = "\t//Class Fields\n".to_owned();
        for (field_name,field) in self.fields(){
            class_fields.push_str(&format!("\t{ctype} {field_name};\n",ctype = field.c_type()));
            if let Some(dep)= field.dependency(){
                includes.push_str(&format!("#include \"{dep}.hpp\"\n"));
            }
        }
        class_fields.push_str("\t//Static fields\n");
        class_fields.push_str(&format!("\tClassData* classData;\n"));
        for (field_name,field) in self.statics_fields(){
            class_fields.push_str(&format!("\tstatic {ctype} {field_name};\n",ctype = field.c_type()));
            if let Some(dep)= field.dependency(){
                includes.push_str(&format!("#include \"{dep}.hpp\"\n"));
            }
        }
        let mut class_methods = "\t//Class Virtual Methods\n".to_owned();
        for (method_name,method) in self.static_methods(){
            class_methods.push_str(&format!("\tstatic {ret} {name}(",ret = method.ret_val().c_type(),name = method.name()));
            let mut margs = method.args().iter();
            match margs.next(){
                Some(arg)=>class_methods.push_str(&arg.c_type()),
                None=>(),
            }
            for marg in margs{
                class_methods.push(',');
                class_methods.push_str(&marg.c_type());
                if let Some(dep)= marg.dependency(){
                    includes.push_str(&format!("#include \"{dep}.hpp\"\n"));
                }
            }
            class_methods.push_str(");\n");
        }
        for (method_name,method) in self.virtual_methods(){
            class_methods.push_str(&format!("\tvirtual {ret} {name}(",ret = method.ret_val().c_type(),name = method.name()));
            let mut margs = method.args().iter();
            //println!("\n\t{name}::{method_name}->{margs:?}",name = self.name);
            match margs.next(){
                Some(arg)=>class_methods.push_str(&arg.c_type()),
                None=>(),
            }
            for marg in margs{
                class_methods.push(',');
                class_methods.push_str(&marg.c_type());
                if let Some(dep)= marg.dependency(){
                    includes.push_str(&format!("#include \"{dep}.hpp\"\n"));
                }
            }
            class_methods.push_str(");\n");
        }
        class_methods.push_str("\t//Class Static methods\n");
        let class_ifaces = "".to_owned();
        let class_def = format!("struct {class_name}:{super_name}{{\n{class_fields}{class_methods}}};");
        let final_header = format!("#pragma once\n#ifndef {class_name}_INCLUDE_GUARD\n#define {class_name}_INCLUDE_GUARD\n{includes}\n{class_def}\n#endif"); 
        hout.write_all(final_header.as_bytes())
    }
}
