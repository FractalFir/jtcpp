use crate::importer::{ImportedJavaClass,opcodes::OpCode};
use crate::IString;
use crate::{mangle_method_name,method_desc_to_argc};
#[derive(Debug)]
enum FieldType{
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
fn fieldref_to_info(index:u16,class:&ImportedJavaClass)->(FieldType,IString,IString){
    let (field_class,nametype) = class.lookup_filed_ref(index).unwrap();
    let field_class = class.lookup_class(field_class).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let ftype = match descriptor.chars().nth(0).unwrap(){
        'B'=>FieldType::Byte,
        'C'=>FieldType::Char,
        'D'=>FieldType::Double,
        'F'=>FieldType::Float,
        'I'=>FieldType::Int,
        'J'=>FieldType::Long,
        'L' | '[' =>FieldType::ObjectRef,
        'S' =>FieldType::Short,
        'Z' => FieldType::Bool,
        _=>panic!("Invalid field descriptor!\"{descriptor}\""),
    };
    (ftype,field_class.into(),name.into())
}
fn methodref_to_mangled_and_argc(index:u16,class:&ImportedJavaClass)->(IString,u8){
    let (method_class, nametype) = class.lookup_method_ref(index).unwrap();
    let (name, descriptor) = class.lookup_nametype(nametype).unwrap();
    let method_class = class.lookup_class(method_class).unwrap();
    let name = class.lookup_utf8(name).unwrap();
    let descriptor = class.lookup_utf8(descriptor).unwrap();
    let mangled = mangle_method_name(method_class, name, descriptor);
    //let method_id = self.code_container.lookup_or_insert_method(&mangled);
    let argc = method_desc_to_argc(&descriptor);
    (mangled,argc)
}
#[derive(Debug)]
pub(crate) enum FatOp{
    IConst(i32),
    LConst(i32),
    SConst(IString),
    FConst(f32),
    DConst(f64),
    ALoad(u8),
    FLoad(u8),
    DLoad(u8),
    ILoad(u8),
    IStore(u8),
    FStore(u8),
    IAdd,
    FAdd,
    FSub,
    IMul,
    FMul,
    ISub,
    IRem,
    IDiv,
    InvokeSpecial(IString,u8),
    InvokeStatic(IString,u8),
    InvokeVirtual(IString,u8),
    ZGetStatic(IString,IString),
    BGetStatic(IString,IString),
    SGetStatic(IString,IString),
    IGetStatic(IString,IString),
    LGetStatic(IString,IString),
    FGetStatic(IString,IString),
    DGetStatic(IString,IString),
    OGetStatic(IString,IString),
    CGetStatic(IString,IString),
    ZGetField(IString,IString),
    BGetField(IString,IString),
    SGetField(IString,IString),
    IGetField(IString,IString),
    LGetField(IString,IString),
    FGetField(IString,IString),
    DGetField(IString,IString),
    OGetField(IString,IString),
    CGetField(IString,IString),
    ZPutField(IString,IString),
    BPutField(IString,IString),
    SPutField(IString,IString),
    IPutField(IString,IString),
    LPutField(IString,IString),
    FPutField(IString,IString),
    DPutField(IString,IString),
    OPutField(IString,IString),
    CPutField(IString,IString),
    Dup,
    Return,
    FReturn,
    IReturn,
}
pub(crate) fn expand_ops(ops:&[(OpCode,u16)],class:&ImportedJavaClass)->Box<[FatOp]>{
    let mut fatops = Vec::with_capacity(ops.len());
    for op in ops{
        let cop = match op.0{
            OpCode::LoadConst(index) => {
                let const_item = class.lookup_item(index).unwrap();
                match const_item{
                    crate::importer::ConstantItem::ConstString { string_index}=>{
                        let string = class.lookup_utf8(*string_index).unwrap();
                        FatOp::SConst(string.into())
                    },
                    _=>todo!("can't handle const!{const_item:?}"),
                }
            },
            OpCode::IConst(int) => FatOp::IConst(int),
            OpCode::ISub=>FatOp::ISub,
            OpCode::FSub=>FatOp::FSub,
            OpCode::IAdd=>FatOp::IAdd,
            OpCode::FAdd=>FatOp::FAdd,
            OpCode::IMul=>FatOp::IMul,
            OpCode::FMul=>FatOp::FMul,
            OpCode::IDiv=>FatOp::IDiv,
            OpCode::IRem=>FatOp::IRem,
            OpCode::ALoad(index) => FatOp::ALoad(index),
            OpCode::ILoad(index) => FatOp::ILoad(index),
            OpCode::IStore(index) => FatOp::IStore(index),
            OpCode::FStore(index) => FatOp::FStore(index),
            OpCode::FLoad(index) => FatOp::FLoad(index),
            OpCode::GetStatic(index) =>{
                let (ftype,class,name) = fieldref_to_info(index,class);
                match ftype{
                    FieldType::Bool=>FatOp::ZGetStatic(class,name),
                    FieldType::Byte=>FatOp::BGetStatic(class,name),
                    FieldType::Short=>FatOp::SGetStatic(class,name),
                    FieldType::Char => FatOp::CGetStatic(class,name),
                    FieldType::Int=>FatOp::IGetStatic(class,name),
                    FieldType::Long=>FatOp::LGetStatic(class,name),
                    FieldType::Float=>FatOp::FGetStatic(class,name),
                    FieldType::Double=>FatOp::DGetStatic(class,name),
                    FieldType::ObjectRef=>FatOp::OGetStatic(class,name),
                }
            }
            OpCode::GetField(index) =>{
                let (ftype,class,name) = fieldref_to_info(index,class);
                match ftype{
                    FieldType::Bool=>FatOp::ZGetField(class,name),
                    FieldType::Byte=>FatOp::BGetField(class,name),
                    FieldType::Short=>FatOp::SGetField(class,name),
                    FieldType::Char => FatOp::CGetField(class,name),
                    FieldType::Int=>FatOp::IGetField(class,name),
                    FieldType::Long=>FatOp::LGetField(class,name),
                    FieldType::Float=>FatOp::FGetField(class,name),
                    FieldType::Double=>FatOp::DGetField(class,name),
                    FieldType::ObjectRef=>FatOp::OGetField(class,name),
                }
            }
            OpCode::PutField(index) =>{
                let (ftype,class,name) = fieldref_to_info(index,class);
                match ftype{
                    FieldType::Bool=>FatOp::ZPutField(class,name),
                    FieldType::Byte=>FatOp::BPutField(class,name),
                    FieldType::Short=>FatOp::SPutField(class,name),
                    FieldType::Char => FatOp::CPutField(class,name),
                    FieldType::Int=>FatOp::IPutField(class,name),
                    FieldType::Long=>FatOp::LPutField(class,name),
                    FieldType::Float=>FatOp::FPutField(class,name),
                    FieldType::Double=>FatOp::DPutField(class,name),
                    FieldType::ObjectRef=>FatOp::OPutField(class,name),
                }
            }
            OpCode::Dup => FatOp::Dup,
            ///TODO: handle non-static methods(change argc by 1)
            OpCode::InvokeSpecial(index) =>{
                let (name,argc) = methodref_to_mangled_and_argc(index,class);
                FatOp::InvokeSpecial(name,argc)
            },
            OpCode::InvokeStatic(index) =>{
                let (name,argc) = methodref_to_mangled_and_argc(index,class);
                FatOp::InvokeStatic(name,argc)
            },
            OpCode::InvokeVirtual(index) =>{
                let (name,argc) = methodref_to_mangled_and_argc(index,class);
                FatOp::InvokeVirtual(name,argc)
            },
            OpCode::Return => FatOp::Return,
            OpCode::FReturn => FatOp::FReturn,
            OpCode::IReturn => FatOp::IReturn,
            _=>todo!("can't expand op {op:?}"),
        };
        fatops.push(cop);
    }
    fatops.into()
}

