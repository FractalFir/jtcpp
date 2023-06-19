use crate::IString;
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
load_fn_impl!(load_u8, u8);
#[derive(Debug)]
pub(crate) struct Field{
    name_index:u16,
    descriptor_index:u16,
    attributes:Box<[Attribute]>
}
impl Field {
    fn read<R: std::io::Read>(src: &mut R, const_items:&[ConstantItem]) -> Result<Self, std::io::Error> {
        let flags = AccessFlags::read(src)?;
        let name_index = load_u16(src)?;
        let descriptor_index = load_u16(src)?;
        let attributes_count = load_u16(src)?;
        let mut attributes = Vec::with_capacity(attributes_count as usize);
        for _ in 0..attributes_count {
            attributes.push(Attribute::read(src, const_items)?);
        }
        Ok(Self{name_index,descriptor_index, attributes: attributes.into()})
    }
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum OpCode {
    ALoad(u8),
    ILoad(u8),
    IStore(u8),
    IAdd,
    ISub,
    IMul,
    IDiv,
    IRem,
    InvokeSpecial(u16),
    InvokeVirtual(u16),
    InvokeStatic(u16),
    Return,
    IReturn,
    GetStatic(u16),
    LoadConst(u16),
}
fn load_ops<R: std::io::Read>(
    src: &mut R,
    code_length: u32,
) -> Result<Vec<OpCode>, std::io::Error> {
    let mut curr_offset = 0;
    let mut ops = Vec::with_capacity(code_length as usize);
    while curr_offset < code_length {
        let op = load_u8(src)?;
        print!("{curr_offset}");
        curr_offset += 1;
        let decoded_op = match op {
            0x12 => {
                let constant_pool_index = load_u8(src)?;
                curr_offset += 1;
                OpCode::LoadConst(constant_pool_index as u16)
            }
            0x13 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::LoadConst(constant_pool_index)
            }
            0x15 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::ILoad(index)
            }
            0x1a..=0x1d => OpCode::ILoad(op - 0x1a),
            0x2a..=0x2d => OpCode::ALoad(op - 0x2a),
            0x3b..=0x3e => OpCode::IStore(op - 0x3b),
            0x36 => {
                let index = load_u8(src)?;
                curr_offset += 1;
                OpCode::IStore(index)
            }
            0x60 => OpCode::IAdd,
            0x64 => OpCode::ISub,
            0x68 => OpCode::IMul,
            0x6c => OpCode::IDiv,
            0x70 => OpCode::IRem,
            0xac => OpCode::IReturn,
            0xb1 => OpCode::Return,
            0xb2 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::GetStatic(constant_pool_index)
            }
            0xb6 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeVirtual(constant_pool_index)
            }
            0xb7 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeSpecial(constant_pool_index)
            }
            0xb8 => {
                let constant_pool_index = load_u16(src)?;
                curr_offset += 2;
                OpCode::InvokeStatic(constant_pool_index)
            }
            _ => todo!("Unhandled opcode:{op:x}!"),
        };
        ops.push(decoded_op);
        //println!("{decoded_op:?}");
    }
    //println!("ops:{ops:?}");
    Ok(ops)
}
#[derive(Debug)]
enum Attribute {
    Code {
        max_stack: u16,
        max_locals: u16,
        ops: Box<[OpCode]>,
        attributes: Box<[Attribute]>,
    },
    LineNumberTable {
        pc_lines: Box<[(u16, u16)]>,
    },
    SourceFile {
        sourcefile_index: u16,
    },
}
impl Attribute {
    fn decode_attribute<R: std::io::Read>(
        src: &mut R,
        attribute_name: &str,
        const_items: &[ConstantItem],
    ) -> Result<Self, std::io::Error> {
        match attribute_name {
            "LineNumberTable" => {
                let line_number_table_length = load_u16(src)?;
                let mut pc_lines = Vec::with_capacity(line_number_table_length as usize);
                for _ in 0..line_number_table_length {
                    let start_pc = load_u16(src)?;
                    let line_number = load_u16(src)?;
                    pc_lines.push((start_pc, line_number));
                }
                Ok(Self::LineNumberTable {
                    pc_lines: pc_lines.into(),
                })
            }
            "SourceFile" => {
                let sourcefile_index = load_u16(src)?;
                Ok(Self::SourceFile { sourcefile_index })
            }
            "Code" => {
                let max_stack = load_u16(src)?;
                let max_locals = load_u16(src)?;
                let code_length = load_u32(src)?;
                let ops = load_ops(src, code_length)?;
                let exception_table_length = load_u16(src)?;
                assert_eq!(exception_table_length, 0, "Exceptions not supported yet!");
                let attributes_count = load_u16(src)?;
                let mut attributes = Vec::with_capacity(attributes_count as usize);
                for _ in 0..attributes_count {
                    attributes.push(Self::read(src, const_items)?);
                }
                Ok(Self::Code {
                    max_stack,
                    max_locals,
                    ops: ops.into(),
                    attributes: attributes.into(),
                })
            }
            _ => todo!("Can't read attributes of type {attribute_name}!"),
        }
    }
    fn read<R: std::io::Read>(
        src: &mut R,
        const_items: &[ConstantItem],
    ) -> Result<Self, std::io::Error> {
        let attribute_name_index = load_u16(src)?;
        assert!(attribute_name_index > 0);
        let attribute_name = &const_items[(attribute_name_index - 1) as usize];
        let attribute_name = if let ConstantItem::Utf8(attribute_name) = attribute_name {
            attribute_name
        } else {
            panic!("Atribute name must be a UTF8 string!");
        };
        let attribute_length = load_u32(src)? as usize;
        let mut attibute_data = vec![0; attribute_length];
        src.read(&mut attibute_data)?;
        Self::decode_attribute(&mut &attibute_data[..], attribute_name, const_items)
    }
}
pub(crate) struct Method {
    access_flags: AccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Box<[Attribute]>,
}
impl Method {
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
    pub(crate) fn bytecode(&self) -> Option<&[OpCode]> {
        for attribute in self.attributes.iter() {
            if let Attribute::Code {
                ops,
                max_stack,
                max_locals,
                attributes,
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
enum ConstantItem {
    MethodRef {
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
    Utf8(IString),
}
#[derive(Debug)]
pub enum ConstantImportError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
}
struct AccessFlags {
    mask: u16,
}
impl AccessFlags {
    fn read<R: std::io::Read>(src: &mut R) -> Result<Self, std::io::Error> {
        let mask = load_u16(src)?;
        Ok(Self { mask })
    }
    fn is_public(&self) -> bool {
        self.mask & 0x0001 != 0
    }
    fn is_final(&self) -> bool {
        self.mask & 0x0010 != 0
    }
    fn is_super(&self) -> bool {
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
        match tag {
            1 => {
                let length = load_u16(src)?;
                let mut bytes = vec![0; length as usize];
                src.read_exact(&mut bytes)?;
                let istring: IString = std::str::from_utf8(&bytes)?.to_owned().into_boxed_str();
                Ok(Self::Utf8(istring))
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
            12 => {
                let name_index = load_u16(src)?;
                let descriptor_index = load_u16(src)?;
                Ok(Self::NameAndType {
                    name_index,
                    descriptor_index,
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
    if major != 64 || minor != 0 {
        return Err(BytecodeImportError::UnsuportedVersion(major, minor));
    }
    let constant_pool_count = load_u16(src)?;
    let mut const_items = Vec::with_capacity(constant_pool_count as usize);
    for _ in 0..(constant_pool_count - 1) {
        const_items.push(ConstantItem::read(src)?);
    }
    let access_flags = AccessFlags::read(src)?;
    let this_class = load_u16(src)?;
    if this_class < 0 || this_class > constant_pool_count {
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
        fields.push(Field::read(src,&const_items)?);
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
#[test]
fn load_ident_class() {
    let mut file = std::fs::File::open("test/Identity.class").unwrap();
    let class = load_class(&mut file).unwrap();
}
