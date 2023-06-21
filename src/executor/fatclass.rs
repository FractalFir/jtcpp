use super::{field_descriptor_to_ftype, FieldType};
use crate::importer::ImportedJavaClass;
use crate::IString;
pub(crate) struct FatClass {
    class_name: IString,
    super_name: IString,
    fields: Vec<(IString, FieldType)>,
    static_fields: Vec<(IString, FieldType)>,
}
impl FatClass {
    pub(crate) fn class_name(&self) -> &str {
        &self.class_name
    }
    pub(crate) fn super_name(&self) -> &str {
        &self.super_name
    }
    pub(crate) fn static_fields(&self) -> &[(IString, FieldType)]{
        &self.static_fields
    }
    pub(crate) fn fields(&self) -> &[(IString, FieldType)]{
        &self.fields
    }
}
pub(crate) fn expand_class(class: &ImportedJavaClass) -> FatClass {
    let this_class = class.this_class();
    let class_name: IString = class.lookup_class(this_class).unwrap().into();
    let super_class = class.super_class();
    let super_name: IString = class.lookup_class(super_class).unwrap().into();
    let mut fields: Vec<(IString, FieldType)> = Vec::with_capacity(class.fields().len());
    let mut static_fields: Vec<(IString, FieldType)> = Vec::with_capacity(class.fields().len());
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
        static_fields,
    }
}
