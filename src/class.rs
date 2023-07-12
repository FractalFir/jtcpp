use crate::{
    class_path_to_class_mangled, field_descriptor_to_ftype, IString, Method, VariableType,
    ERR_SUPER_INVALID, ERR_THIS_INVALID,
};
pub(crate) struct Class {
    name: IString,
    parrent: IString,
    fields: Vec<(IString, VariableType)>,
    static_fields: Vec<(IString, VariableType)>,
    static_methods: Vec<(IString, Method)>,
    virtual_methods: Vec<(IString, Method)>,
}
impl Class {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn parrent_name(&self) -> &str {
        &self.parrent
    }
    pub(crate) fn static_methods(&self) -> &[(IString, Method)] {
        &self.static_methods
    }
    pub(crate) fn virtual_methods(&self) -> &[(IString, Method)] {
        &self.virtual_methods
    }
    pub(crate) fn fields(&self) -> &[(IString, VariableType)] {
        &self.fields
    }
    pub(crate) fn static_fields(&self) -> &[(IString, VariableType)] {
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
            if method.is_virtual(java_class) || method.name(java_class).contains("<init>") {
                let mangled_name = method.virtual_name(java_class);
                let method = Method::from_raw_method(method, &mangled_name, java_class);
                virtual_methods.push((mangled_name, method));
            } else {
                let mangled_name = method.mangled_name(java_class);
                let method = Method::from_raw_method(method, &mangled_name, java_class);
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
}
