pub(crate) mod baseir;
pub(crate) mod class;
pub(crate) mod fatclass;
pub(crate) mod fatops;
use crate::importer::ImportedJavaClass;
use crate::{Value,IString};
#[derive(Debug)]
pub(crate) enum UnmetDependency {
    NeedsClass(IString),
}
#[derive(Debug,Clone)]
pub(crate) enum FieldType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    ObjectRef,
    Bool,
}
impl FieldType {
    fn default_value(&self) -> Value {
        match self {
            Self::Int => Value::Int(0),
            Self::Float => Value::Float(0.0),
            Self::ObjectRef => Value::ObjectRef(0),
            _ => todo!("Can't create default value of type {self:?}"),
        }
    }
}
pub(crate) fn field_descriptor_to_ftype(descriptor: u16, class: &ImportedJavaClass) -> FieldType {
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    match descriptor.chars().nth(0).unwrap() {
        'B' => FieldType::Byte,
        'C' => FieldType::Char,
        'D' => FieldType::Double,
        'F' => FieldType::Float,
        'I' => FieldType::Int,
        'J' => FieldType::Long,
        'L' | '[' => FieldType::ObjectRef,
        'S' => FieldType::Short,
        'Z' => FieldType::Bool,
        _ => panic!("Invalid field descriptor!\"{descriptor}\""),
    }
}
