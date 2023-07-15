use crate::{
    field_descriptor_to_ftype, IString, Method, VariableType, ERR_SUPER_INVALID, ERR_THIS_INVALID,
};
pub(crate) struct Class {
    name: IString,
    parrent: IString,
    ifaces: Box<[IString]>,
    fields: Vec<(IString, VariableType)>,
    static_fields: Vec<(IString, VariableType)>,
    static_methods: Vec<(IString, Method)>,
    virtual_methods: Vec<(IString, Method)>,
}
pub fn java_class_to_cpp_class(path: &str) -> IString {
    path.replace("/", "::").into()
}
pub fn cpp_class_to_path(class: &str) -> IString {
    class.replace("::", "_cs_").into()
}
#[test]
fn java_to_cpp_paths() {
    assert_eq!(
        &*java_class_to_cpp_class("java/lang/Object"),
        "java::lang::Object"
    );
    assert_eq!(&*java_class_to_cpp_class("Vector3"), "Vector3");
    assert_eq!(
        &*cpp_class_to_path(&*java_class_to_cpp_class("java/lang/Object")),
        "java_cs_lang_cs_Object"
    );
}
impl Class {
    pub(crate) fn path(&self) -> IString {
        cpp_class_to_path(self.cpp_name())
    }
    pub(crate) fn cpp_name(&self) -> &str {
        &self.name
    }
    pub(crate) fn parrent_cpp_name(&self) -> &str {
        &self.parrent
    }
    pub(crate) fn parrent_path(&self) -> IString {
        cpp_class_to_path(&self.parrent)
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
        let class_name = java_class_to_cpp_class(name);
        let parrent = match java_class.lookup_class(java_class.super_class()) {
            Some(parrent) => parrent,
            None => {
                eprintln!(
                    "\nMalformed class file! `super_class` field did not refer to a valid class!"
                );
                std::process::exit(ERR_SUPER_INVALID);
            }
        };
        let parrent = java_class_to_cpp_class(parrent);
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
            if method.is_virtual(java_class) {
                let mangled_name = crate::mangle_method_name(
                    method.name(java_class),
                    method.descriptor(java_class),
                );
                let method = Method::from_raw_method(method, &mangled_name, java_class);
                virtual_methods.push((mangled_name.into(), method));
            } else {
                let mangled_name = crate::mangle_method_name(
                    method.name(java_class),
                    method.descriptor(java_class),
                );
                let method = Method::from_raw_method(method, &mangled_name, java_class);
                static_methods.push((mangled_name.into(), method));
            }
        }
        let ifaces = Vec::new();
        Class {
            name: class_name,
            parrent,
            fields,
            ifaces: ifaces.into(),
            static_fields,
            static_methods,
            virtual_methods,
        }
    }
}
