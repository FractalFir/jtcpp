#![allow(dead_code)]
mod attribute;
pub mod opcodes;
use crate::importer::attribute::BootstrapMethod;
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
//load_fn_impl!(load_u64, u64);
load_fn_impl!(load_i64, i64);
load_fn_impl!(load_f64, f64);
load_fn_impl!(load_i32, i32);
load_fn_impl!(load_u32, u32);
load_fn_impl!(load_f32, f32);
load_fn_impl!(load_u16, u16);
load_fn_impl!(load_i16, i16);
load_fn_impl!(load_u8, u8);
load_fn_impl!(load_i8, i8);
#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) flags: AccessFlags,
    pub(crate) name_index: u16,
    pub(crate) descriptor_index: u16,
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
            flags,
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
    pub(crate) fn name<'a>(&'a self, class: &'a ImportedJavaClass) -> &str {
        class.lookup_utf8(self.name_index).unwrap()
    }
    pub(crate) fn descriptor<'a>(&'a self, class: &'a ImportedJavaClass) -> &str {
        class.lookup_utf8(self.descriptor_index).unwrap()
    }
    pub(crate) fn is_virtual(&self, class: &ImportedJavaClass) -> bool {
        !(self.access_flags.is_static() || self.name(class).contains("init"))
    }
    /*
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
                ops: _,
                max_stack: _,
                max_locals,
                attributes: _,
                exceptions: _,
            } = attribute
            {
                return Some(*max_locals);
            };
        }
        None
    }*/
    pub(crate) fn bytecode(&self) -> Option<&[(OpCode, u16)]> {
        for attribute in self.attributes.iter() {
            if let Attribute::Code {
                ops,
                max_stack: _,
                max_locals: _,
                attributes: _,
                exceptions: _,
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
    super_class: u16,
    fields: Box<[Field]>,
    methods: Box<[Method]>,
    interfaces: Box<[u16]>,
    attributes: Box<[Attribute]>, //field_names: Box<[IString]>,
}
impl ImportedJavaClass {
    pub(crate) fn interfaces(&self) -> &[u16] {
        &self.interfaces
    }
    pub(crate) fn lookup_invoke_dynamic(&self, dynamic: u16) -> Option<(u16, u16)> {
        let dynamic = &self.const_items[dynamic as usize - 1];
        if let ConstantItem::InvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        } = dynamic
        {
            Some((*bootstrap_method_attr_index, *name_and_type_index))
        } else {
            None
        }
    }
    pub(crate) fn lookup_method_handle(&self, method_handle: u16) -> Option<(u8, u16)> {
        let method_handle = &self.const_items[method_handle as usize - 1];
        if let ConstantItem::MethodHandle {
            reference_kind,
            reference_index,
        } = method_handle
        {
            Some((*reference_kind, *reference_index))
        } else {
            None
        }
    }
    pub(crate) fn lookup_bootstrap_method(&self, index: u16) -> Option<&BootstrapMethod> {
        for attribute in self.attributes.iter() {
            if let Attribute::BootstrapMethods { bootstrap_methods } = attribute {
                return bootstrap_methods.get(index as usize);
            }
        }
        None
    }
    pub(crate) fn name(&self) -> &str {
        self.lookup_class(self.this_class).unwrap()
    }
    pub(crate) fn this_class(&self) -> u16 {
        self.this_class
    }
    pub(crate) fn super_class(&self) -> u16 {
        self.super_class
    }
    pub(crate) fn lookup_item(&self, index: u16) -> Option<&ConstantItem> {
        if index < 1 || index as usize >= self.const_items.len() {
            None
        } else {
            Some(&self.const_items[index as usize - 1])
        }
    }
    pub(crate) fn lookup_utf8(&self, utf8: u16) -> Option<&str> {
        let utf8 = &self.const_items[utf8 as usize - 1];
        if let ConstantItem::Utf8(string) = utf8 {
            Some(string)
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
        } else if let ConstantItem::InterfaceMethodRef {
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
    Intiger(i32),
    Float(f32),
    Double(f64),
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
        descriptor_index: u16,
    },
    Module {
        name_index: u16,
    },
    Package {
        name_index: u16,
    },
    Utf8(IString),
    Long(i64),
    Padding,
}
impl ConstantItem {
    fn size(&self) -> u16 {
        match self {
            Self::Long(_) | Self::Double(_) => 2,
            _ => 1,
        }
    }
}
#[derive(Debug)]
pub enum ConstantImportError {
    ZeroTypeConstError,
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
    fn is_private(&self) -> bool {
        self.mask & 0x0002 != 0
    }
    fn is_protected(&self) -> bool {
        self.mask & 0x0004 != 0
    }
    pub fn is_static(&self) -> bool {
        self.mask & 0x0008 != 0
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
            0 => Err(ConstantImportError::ZeroTypeConstError),
            1 => {
                let length = load_u16(src)?;
                let mut bytes = vec![0; length as usize];
                src.read_exact(&mut bytes)?;
                let istring: IString = std::str::from_utf8(&bytes)?.to_owned().into_boxed_str();
                //println!("bytes:{bytes:?} string:{istring}");
                Ok(Self::Utf8(istring))
            }
            3 => {
                let int = load_i32(src)?;
                Ok(Self::Intiger(int))
            }
            4 => {
                let float = load_f32(src)?;
                Ok(Self::Float(float))
            }
            5 => {
                let long = load_i64(src)?;
                //println!("long:{long:x}");
                Ok(Self::Long(long))
            }
            6 => {
                let double = load_f64(src)?;
                Ok(Self::Double(double))
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
                Ok(Self::MethodType { descriptor_index })
            }
            18 => {
                let bootstrap_method_attr_index = load_u16(src)?;
                let name_and_type_index = load_u16(src)?;
                Ok(Self::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                })
            }
            19 => {
                let name_index = load_u16(src)?;
                Ok(Self::Module { name_index })
            }
            20 => {
                let name_index = load_u16(src)?;
                Ok(Self::Package { name_index })
            }
            2 | 21.. => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid ConstItem type!",
            )
            .into()),
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
    if !(50..=64).contains(&major) || minor != 0 {
        return Err(BytecodeImportError::UnsuportedVersion(major, minor));
    }
    let constant_pool_count = load_u16(src)?;
    //println!("constant_pool_count:{constant_pool_count:?}");
    let mut const_items = Vec::with_capacity(constant_pool_count as usize);
    let mut curr_item = 1;
    while curr_item < constant_pool_count {
        let ci = ConstantItem::read(src)?;
        //println!("curr_item:{curr_item}\tci:{ci:?}");
        let ci_size = ci.size();
        curr_item += ci_size;
        const_items.push(ci);
        if ci_size == 2 {
            const_items.push(ConstantItem::Padding);
        }
    }
    let _access_flags = AccessFlags::read(src)?;
    //println!("access_flags:{access_flags:?}");
    let this_class = load_u16(src)?;
    //println!("this_class:{this_class}");
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
    //println!("const_items:{const_items:?}");
    Ok(ImportedJavaClass {
        attributes: attributes.into(),
        fields: fields.into(),
        methods: methods.into(),
        interfaces: interfaces.into(),
        const_items: const_items.into(),
        this_class,
        super_class,
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
    src: &mut (impl std::io::Read + std::io::Seek),
) -> Result<Vec<ImportedJavaClass>, BytecodeImportError> {
    let mut zip = zip::ZipArchive::new(src)?;
    let mut classes = Vec::new();
    for i in 0..zip.len() {
        use std::io::Read;
        let mut file = zip.by_index(i)?;
        let file_name = file.name().to_owned();
        let mut tmp = Vec::new();
        file.read_to_end(&mut tmp)?;
        let mut file = std::io::Cursor::new(tmp);
        let ext = file_name.split('.').last();
        let ext = if let Some(ext) = ext { ext } else { continue }.to_owned();
        if ext == "class" {
            //println!("Filename: {}", file.name());
            let loaded = load_class(&mut file);
            match loaded {
                Ok(class) => classes.push(class),
                Err(err) => {
                    use std::io::Seek;
                    let dump_path = format!("target/testres/{}", file_name);

                    std::fs::create_dir_all(dump_path.split('.').next().unwrap())?;
                    println!("{dump_path}");
                    let mut out = std::fs::File::create(dump_path).unwrap();
                    file.rewind().unwrap();
                    //let mut file = zip.by_index(i)?;
                    std::io::copy(&mut file, &mut out).unwrap();
                    println!("Error:\"{err:?}\" while loading {}.", file_name)
                }
            }
        }
        if ext == "jar" {
            //println!("Filename: {}", file.name());
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
    let _class = load_class(&mut file).unwrap();
}
