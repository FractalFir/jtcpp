use super::{field_descriptor_to_ftype, FieldType};
use crate::importer::ImportedJavaClass;

use crate::IString;
pub(crate) struct FatClass {
    virtuals: Vec<(IString, IString)>,
    class_name: IString,
    super_name: IString,
    interfaces: Vec<IString>,
    fields: Vec<(IString, FieldType)>,
    static_fields: Vec<(IString, FieldType)>,
}
impl FatClass {
    pub(crate) fn interfaces(&self) -> &[IString] {
        &self.interfaces
    }
    pub(crate) fn add_virtual(&mut self, virtual_partialy_mangled: &str, method_mangled: &str) {
        self.virtuals
            .push((virtual_partialy_mangled.into(), method_mangled.into()));
    }
    pub(crate) fn add_static(&mut self, name: &str, ftype: FieldType) {
        self.static_fields.push((name.into(), ftype.clone()));
    }
    pub(crate) fn add_field(&mut self, name: &str, ftype: FieldType) {
        self.fields.push((name.into(), ftype.clone()));
    }
    pub(crate) fn new(class_name: &str, super_name: &str) -> Self {
        Self {
            class_name: class_name.into(),
            super_name: super_name.into(),
            fields: Vec::new(),
            virtuals: Vec::new(),
            static_fields: Vec::new(),
            interfaces: Vec::new(),
        }
    }
    pub(crate) fn class_name(&self) -> &str {
        &self.class_name
    }
    pub(crate) fn super_name(&self) -> &str {
        &self.super_name
    }
    pub(crate) fn static_fields(&self) -> &[(IString, FieldType)] {
        &self.static_fields
    }
    pub(crate) fn fields(&self) -> &[(IString, FieldType)] {
        &self.fields
    }
    pub(crate) fn virtuals(&self) -> &[(IString, IString)] {
        &self.virtuals
    }
}
pub(crate) fn expand_class(class: &ImportedJavaClass) -> FatClass {
    let this_class = class.this_class();
    let class_name: IString = class.lookup_class(this_class).unwrap().into();
    let super_class = class.super_class();
    let super_name: IString = class.lookup_class(super_class).unwrap().into();
    let mut fields: Vec<(IString, FieldType)> = Vec::with_capacity(class.fields().len());
    let mut static_fields: Vec<(IString, FieldType)> = Vec::with_capacity(class.fields().len());
    let mut virtuals = Vec::new();
    for method in class.methods() {
        //Not static, so virtual.
        if method.is_virtual(class) {
            let virtual_name = method.virtual_name(class);
            let real_name = method.mangled_name(class);
            //todo!("VNAME:{virtual_name},RNAME:{real_name}");
            virtuals.push((virtual_name, real_name));
        }
    }
    let mut interfaces: Vec<IString> = Vec::with_capacity(class.interfaces().len());
    //println!("interfaces beg");
    for interface in class.interfaces() {
        let iface_class = class.lookup_class(*interface).unwrap();
        interfaces.push(iface_class.into());
        //println!("interface:{interface} iface_class:{iface_class}");
    }
    //println!("interfaces end");
    for field in class.fields() {
        let (name_index, descriptor_index) = (field.name_index, field.descriptor_index);
        let name = class.lookup_utf8(name_index).unwrap();
        let ftype = field_descriptor_to_ftype(descriptor_index, class);
        if field.flags.is_static() {
            static_fields.push((name.into(), ftype));
        } else {
            fields.push((name.into(), ftype));
        }
        //println!("{ftype:?} {name}");
    }
    FatClass {
        class_name,
        super_name,
        fields,
        virtuals,
        interfaces,
        static_fields,
    }
}
