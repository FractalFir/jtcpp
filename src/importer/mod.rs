pub mod opcodes;
mod attribute;
use crate::IString;
use attribute::Attribute;
use opcodes::OpCode;
macro_rules! load_fn_impl {
    ($name:ident,$tpe:ty) => {
        pub(crate) fn $name<R: std::io::Read>(src: &mut R) -> std::io::Result<$tpe> {
            let mut tmp = [0; std::mem::size_of::<$tpe>()];
            src.read_exact(&mut tmp)?;
            Ok(<$tpe>::from_be_bytes(tmp))
        }
    };
}
load_fn_impl!(load_u64, u64);
load_fn_impl!(load_u32, u32);
load_fn_impl!(load_u16, u16);
load_fn_impl!(load_i16, i16);
load_fn_impl!(load_u8, u8);
load_fn_impl!(load_i8, i8);
#[derive(Debug)]
pub(crate) struct Field {
    name_index: u16,
    descriptor_index: u16,
    attributes: Box<[Attribute]>,
}
impl Field {
    fn read<R: std::io::Read>(
        src: &mut R,
        const_items: &[ConstantItem],
    ) -> Result<Self, std::io::Error> {
        let flags = AccessFlags::read(src)?;
        let name_index = load_u16(src)?;
        let descriptor_index = load_u16(src)?;
        let attributes_count = load_u16(src)?;
        let mut attributes = Vec::with_capacity(attributes_count as usize);
        for _ in 0..attributes_count {
            attributes.push(Attribute::read(src, const_items)?);
        }
        Ok(Self {
            name_index,
            descriptor_index,
            attributes: attributes.into(),
        })
    }
}
pub(crate) struct Method {
    access_flags: AccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Box<[Attribute]>,
}
impl Method {
    pub(crate) fn access_flags(&self) -> &AccessFlags {
        &self.access_flags
    }
    pub(crate) fn name_index(&self) -> u16 {
        self.name_index
    }
    pub(crate) fn descriptor_index(&self) -> u16 {
        self.descriptor_index
    }
    pub(crate) fn max_locals(&self) -> Option<u16> {
        for attribute in self.attributes.iter() {
            if let Attribute::Code {
                ops,
                max_stack,
                max_locals,
                attributes,
                exceptions,
            } = attribute
            {
                return Some(*max_locals);
            };
        }
        None
    }
    pub(crate) fn name(&self, class: &ImportedJavaClass) -> &str {
        todo!();
    }
    pub(crate) fn bytecode(&self) -> Option<&[(OpCode,u16)]> {
        for attribute in self.attributes.iter() {
            if let Attribute::Code {
                ops,
                max_stack,
                max_locals,
                attributes,
                exceptions,
            } = attribute
            {
                return Some(ops);
            };
        }
        None
    }
    fn read<R: std::io::Read>(
        src: &mut R,
        const_items: &[ConstantItem],
    ) -> Result<Self, std::io::Error> {
        let access_flags = AccessFlags::read(src)?;
        let name_index = load_u16(src)?;
        let descriptor_index = load_u16(src)?;
        let attributes_count = load_u16(src)?;
        let mut attributes = Vec::with_capacity(attributes_count as usize);
        for _ in 0..attributes_count {
            attributes.push(Attribute::read(src, const_items)?);
        }
        Ok(Self {
            access_flags,
            name_index,
            descriptor_index,
            attributes: attributes.into(),
        })
    }
}
pub(crate) struct ImportedJavaClass {
    const_items: Box<[ConstantItem]>,
    //name: IString,
    this_class: u16,
    fields: Box<[Field]>,
    methods: Box<[Method]>,
    attributes: Box<[Attribute]>, //field_names: Box<[IString]>,
}
impl ImportedJavaClass {
    pub(crate) fn this_class(&self) -> u16 {
        self.this_class
    }
    pub(crate) fn lookup_item(&self,index: u16)->Option<&ConstantItem>{
        if index < 1 || index as usize >= self.const_items.len(){
            None
        }
        else{Some(&self.const_items[index as usize - 1])}
    }
    pub(crate) fn lookup_utf8(&self, utf8: u16) -> Option<&str> {
        let utf8 = &self.const_items[utf8 as usize - 1];
        if let ConstantItem::Utf8(string) = utf8 {
            Some(&string)
        } else {
            None
        }
    }
    pub(crate) fn lookup_class(&self, class_ref: u16) -> Option<&str> {
        //panic!("Const string index must point to a UTF8 const item!")
        let name_index = &self.const_items[class_ref as usize - 1];
        if let ConstantItem::Class { name_index } = name_index {
            self.lookup_utf8(*name_index)
        } else {
            None
        }
    }
    pub(crate) fn lookup_filed_ref(&self, field_ref: u16) -> Option<(u16, u16)> {
        let field_ref = &self.const_items[field_ref as usize - 1];
        if let ConstantItem::FieldRef {
            class_index,
            name_and_type_index,
        } = field_ref
        {
            Some((*class_index, *name_and_type_index))
        } else {
            None
        }
    }
    pub(crate) fn lookup_nametype(&self, nametype: u16) -> Option<(u16, u16)> {
        let nametype = &self.const_items[nametype as usize - 1];
        if let ConstantItem::NameAndType {
            name_index,
            descriptor_index,
        } = nametype
        {
            Some((*name_index, *descriptor_index))
        } else {
            None
        }
    }
    pub(crate) fn lookup_method_ref(&self, method_ref: u16) -> Option<(u16, u16)> {
        let method_ref = &self.const_items[method_ref as usize - 1];
        if let ConstantItem::MethodRef {
            class_index,
            name_and_type_index,
        } = method_ref
        {
            Some((*class_index, *name_and_type_index))
        } else {
            None
        }
    }
    /*
    pub(crate) fn lookup_const_string(&self, const_string: usize) -> Option<&str> {
        println!("const_string_index:{const_string}");
        let const_string = &self.const_items[const_string - 1];
        let string_index = if let ConstantItem::ConstString { string_index } = const_string {
            string_index
        } else {
            println!("const_string:{const_string:?}");
            return None;
        };
        let utf8 = &self.const_items[*string_index as usize - 1];
        if let ConstantItem::Utf8(string) = utf8 {
            Some(&string)
        } else {
            panic!("Const string index must point to a UTF8 const item!")
        }
    }*/
    fn name(&self) -> &str {
        todo!();
    }
    pub(crate) fn methods(&self) -> &[Method] {
        &self.methods
    }
    pub(crate) fn fields(&self) -> &[Field] {
        &self.fields
    }
}
#[derive(Debug)]
pub(crate) enum ConstantItem {
    Unknown,
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    Class {
        name_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    ConstString {
        string_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index:u16,
    },
    Utf8(IString),
    Long(u64),
}
#[derive(Debug)]
pub enum ConstantImportError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
}
#[derive(Debug)]
pub struct AccessFlags {
    mask: u16,
}
impl AccessFlags {
    pub(crate) fn read<R: std::io::Read>(src: &mut R) -> Result<Self, std::io::Error> {
        let mask = load_u16(src)?;
        Ok(Self { mask })
    }
    fn is_public(&self) -> bool {
        self.mask & 0x0001 != 0
    }
    fn is_final(&self) -> bool {
        self.mask & 0x0010 != 0
    }
    pub fn is_super(&self) -> bool {
        self.mask & 0x0020 != 0
    }
    fn is_interface(&self) -> bool {
        self.mask & 0x0200 != 0
    }
    fn is_abstract(&self) -> bool {
        self.mask & 0x0400 != 0
    }
    pub fn is_static(&self) -> bool {
        self.mask & 0x0008 != 0
    }
    fn is_synthetic(&self) -> bool {
        self.mask & 0x1000 != 0
    }
    fn is_annotantion(&self) -> bool {
        self.mask & 0x2000 != 0
    }
    fn is_enum(&self) -> bool {
        self.mask & 0x4000 != 0
    }
}
impl ConstantItem {
    fn read<R: std::io::Read>(src: &mut R) -> Result<Self, ConstantImportError> {
        let tag = load_u8(src)?;
        //println!("tag:{tag}");
        match tag {
            
            1 => {
                let length = load_u16(src)?;
                let mut bytes = vec![0; length as usize];
                src.read_exact(&mut bytes)?;
                let istring: IString = std::str::from_utf8(&bytes)?.to_owned().into_boxed_str();
                //println!("bytes:{bytes:?} string:{istring}");
                Ok(Self::Utf8(istring))
            }
            5=>{
                let long = load_u64(src)?;
                println!("long:{long:x}");
                Ok(Self::Long(long))
            }
            7 => {
                let name_index = load_u16(src)?;
                Ok(Self::Class { name_index })
            }
            8 => {
                let string_index = load_u16(src)?;
                Ok(Self::ConstString { string_index })
            }
            9 => {
                let class_index = load_u16(src)?;
                let name_and_type_index = load_u16(src)?;
                Ok(Self::FieldRef {
                    class_index,
                    name_and_type_index,
                })
            }
            10 => {
                let class_index = load_u16(src)?;
                let name_and_type_index = load_u16(src)?;
                Ok(Self::MethodRef {
                    class_index,
                    name_and_type_index,
                })
            }
            11 => {
                let class_index = load_u16(src)?;
                let name_and_type_index = load_u16(src)?;
                Ok(Self::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                })
            }
            12 => {
                let name_index = load_u16(src)?;
                let descriptor_index = load_u16(src)?;
                Ok(Self::NameAndType {
                    name_index,
                    descriptor_index,
                })
            }
            15 => {
                let reference_kind = load_u8(src)?;
                let reference_index = load_u16(src)?;
                Ok(Self::MethodHandle {
                    reference_kind,
                    reference_index,
                })
            }
            16 => {
                let descriptor_index = load_u16(src)?;
                Ok(Self::MethodType {
                    descriptor_index
                })
            }
            18 => {
                let bootstrap_method_attr_index = load_u16(src)?;
                let name_and_type_index = load_u16(src)?;
                Ok(Self::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                })
            }
            _ => todo!("Unhandled const info kind: {tag}"),
        }
    }
}
pub(crate) fn load_class<R: std::io::Read>(
    src: &mut R,
) -> Result<ImportedJavaClass, BytecodeImportError> {
    const CLASS_MAGIC: u32 = 0xCAFEBABE;
    let magic = load_u32(src)?;
    if magic != CLASS_MAGIC {
        //println!(
        return Err(BytecodeImportError::NotJavaBytecode(magic));
    }
    let minor = load_u16(src)?;
    let major = load_u16(src)?;
    if major < 50 || major > 64 || minor != 0 {
        return Err(BytecodeImportError::UnsuportedVersion(major, minor));
    }
    let constant_pool_count = load_u16(src)?;
    let mut const_items = Vec::with_capacity(constant_pool_count as usize);
    for _ in 0..(constant_pool_count - 1) {
        const_items.push(ConstantItem::read(src)?);
    }
    let access_flags = AccessFlags::read(src)?;
    let this_class = load_u16(src)?;
    if this_class < 1 || this_class > constant_pool_count {
        return Err(BytecodeImportError::InvalidThisClass);
    }
    let super_class = load_u16(src)?;
    if super_class > constant_pool_count {
        return Err(BytecodeImportError::InvalidSuperClass);
    }
    let interfaces_count = load_u16(src)?;
    let mut interfaces = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        interfaces.push(load_u16(src)?);
    }
    let fields_count = load_u16(src)?;
    let mut fields = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        fields.push(Field::read(src, &const_items)?);
    }
    let methods_count = load_u16(src)?;
    let mut methods = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        methods.push(Method::read(src, &const_items)?);
    }
    let attributes_count = load_u16(src)?;
    let mut attributes = Vec::with_capacity(attributes_count as usize);
    for _ in 0..attributes_count {
        attributes.push(Attribute::read(src, &const_items)?);
    }
    println!("const_items:{const_items:?}");
    Ok(ImportedJavaClass {
        attributes: attributes.into(),
        fields: fields.into(),
        methods: methods.into(),
        const_items: const_items.into(),
        this_class,
    })
}
#[derive(Debug)]
pub enum BytecodeImportError {
    NotJavaBytecode(u32),
    IoError(std::io::Error),
    UnsuportedVersion(u16, u16),
    ConstantImportError(ConstantImportError),
    InvalidThisClass,
    InvalidSuperClass,
    ZipError(zip::result::ZipError),
}
impl From<std::io::Error> for BytecodeImportError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
impl From<std::io::Error> for ConstantImportError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
impl From<std::str::Utf8Error> for ConstantImportError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}
impl From<ConstantImportError> for BytecodeImportError {
    fn from(err: ConstantImportError) -> Self {
        match err {
            ConstantImportError::IoError(err) => Self::IoError(err),
            _ => Self::ConstantImportError(err),
        }
    }
}
impl From<zip::result::ZipError> for BytecodeImportError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::ZipError(err)
    }
}
pub(crate) fn load_jar(
    src: &mut (impl std::io::Read  + std::io::Seek),
) -> Result<Vec<ImportedJavaClass>, BytecodeImportError> {
    let mut zip = zip::ZipArchive::new(src)?;
    let mut classes = Vec::new();
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let ext = file.name().split('.').last();
        let ext = if let Some(ext) = ext{ext} else {continue}.to_owned();
        println!("Filename: {} ext:{ext:?}", file.name());
        if ext == "class" {
            classes.push(load_class(&mut file)?);
        }
        if ext == "jar" {
            use std::io::Read;
            // TODO fix this stupidness, may need to write an issue to request ZipFile to implement Seek.
            let mut tmp = Vec::new();
            file.read_to_end(&mut tmp)?;
            let mut reader = std::io::Cursor::new(tmp); 
            classes.extend(load_jar(&mut reader)?);
        }
    }
    Ok(classes)
}
#[test]
fn load_ident_class() {
    let mut file = std::fs::File::open("test/Identity.class").unwrap();
    let class = load_class(&mut file).unwrap();
}
