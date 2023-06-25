use super::opcodes::{load_ops, OpCode};
use super::{load_i16, load_i8, load_u16, load_u32, load_u64, load_u8, AccessFlags, ConstantItem};
#[derive(Debug)]
pub(crate) struct LocalVariable {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16,
}
#[derive(Debug)]
pub(crate) struct CodeException {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}
#[derive(Debug)]
pub(crate) struct MethodParameter {
    name_index: u16,
    access_flags: AccessFlags,
}
#[derive(Debug)]
pub(crate) struct BootstrapMethod {
    bootstrap_method_ref: u16,
    bootstrap_args: Box<[u16]>,
}
#[derive(Debug)]
pub(crate) enum Attribute {
    Unknown,
    Code {
        max_stack: u16,
        max_locals: u16,
        ops: Box<[(OpCode, u16)]>,
        exceptions: Box<[CodeException]>,
        attributes: Box<[Attribute]>,
    },
    LineNumberTable {
        pc_lines: Box<[(u16, u16)]>,
    },
    SourceFile {
        sourcefile_index: u16,
    },
    LocalVariableTable {
        local_vars: Box<[LocalVariable]>,
    },
    NestHost {
        host_class_index: u16,
    },
    NestMembers {
        classes: Box<[u16]>,
    },
    MethodParameters {
        parameters: Box<[MethodParameter]>,
    },
    BootstrapMethods {
        bootstrap_methods: Box<[BootstrapMethod]>,
    },
    Exceptions {
        exceptions: Box<[u16]>,
    },
    Signature {
        signature: u16,
    },
    ConstantValue{
        value_index:u16
    },
    EnclosingMethod{
        class_index:u16,
        method_index:u16,
    },
    Deprecated,
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
            "LocalVariableTable" => {
                let length = load_u16(src)? as usize;
                let mut local_vars = Vec::with_capacity(length);
                for _ in 0..length {
                    let start_pc = load_u16(src)?;
                    let length = load_u16(src)?;
                    let name_index = load_u16(src)?;
                    let descriptor_index = load_u16(src)?;
                    let index = load_u16(src)?;
                    local_vars.push(LocalVariable {
                        start_pc,
                        length,
                        name_index,
                        descriptor_index,
                        index,
                    })
                }
                Ok(Self::LocalVariableTable {
                    local_vars: local_vars.into(),
                })
            }
            "MethodParameters" => {
                let parameters_count = load_u8(src)? as usize;
                let mut parameters = Vec::with_capacity(parameters_count);
                for _ in 0..parameters_count {
                    let name_index = load_u16(src)?;
                    let access_flags = AccessFlags::read(src)?;
                    parameters.push(MethodParameter {
                        name_index,
                        access_flags,
                    });
                }
                Ok(Self::MethodParameters {
                    parameters: parameters.into(),
                })
            }
            "Deprecated" => Ok(Self::Deprecated),
            "Record" => Ok(Self::Unknown),        // IDK what it does.
            "StackMapTable" => Ok(Self::Unknown), //Not worth the effort.
            "RuntimeVisibleAnnotations" => Ok(Self::Unknown), //TODO: Handle this at some point.
            "LocalVariableTypeTable" => Ok(Self::Unknown), //TODO: Not needed, but nice to have.
            "InnerClasses" => Ok(Self::Unknown),  //TODO: Handle inner classes!
            "Exceptions" => {
                let number_exceptions = load_u16(src)? as usize;
                let mut exceptions = Vec::with_capacity(number_exceptions);
                for _ in 0..number_exceptions {
                    exceptions.push(load_u16(src)?);
                }
                Ok(Self::Exceptions {
                    exceptions: exceptions.into(),
                })
            }
            "Signature" => {
                let signature = load_u16(src)?;
                Ok(Self::Signature { signature })
            }
            "BootstrapMethods" => {
                let bootstrap_method_count = load_u16(src)? as usize;
                let mut bootstrap_methods = Vec::with_capacity(bootstrap_method_count);
                for _ in 0..bootstrap_method_count {
                    let bootstrap_method_ref = load_u16(src)?;
                    let num_bootstrap_arguments = load_u16(src)? as usize;
                    let mut bootstrap_args = Vec::with_capacity(num_bootstrap_arguments);
                    for _ in 0..num_bootstrap_arguments {
                        bootstrap_args.push(load_u16(src)?);
                    }
                    bootstrap_methods.push(BootstrapMethod {
                        bootstrap_method_ref,
                        bootstrap_args: bootstrap_args.into(),
                    });
                }
                Ok(Self::BootstrapMethods {
                    bootstrap_methods: bootstrap_methods.into(),
                })
            }
            "NestHost" => {
                let host_class_index = load_u16(src)?;
                Ok(Self::NestHost { host_class_index })
            }
            "NestMembers" => {
                let class_count = load_u16(src)? as usize;
                let mut classes = Vec::with_capacity(class_count);
                for _ in 0..class_count {
                    classes.push(load_u16(src)?);
                }
                Ok(Self::NestMembers {
                    classes: classes.into(),
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
                let exception_table_length = load_u16(src)? as usize;
                //'assert_eq!(exception_table_length, 0, "Exceptions not supported yet!");
                let mut exceptions = Vec::with_capacity(exception_table_length);
                for _ in 0..exception_table_length {
                    let start_pc = load_u16(src)?;
                    let end_pc = load_u16(src)?;
                    let handler_pc = load_u16(src)?;
                    let catch_type = load_u16(src)?;
                    exceptions.push(CodeException {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type,
                    });
                }
                //println!("exceptions :{exceptions:?}");
                let attributes_count = load_u16(src)?;
                //println!("ac:{attributes_count}");
                let mut attributes = Vec::with_capacity(attributes_count as usize);
                for _ in 0..attributes_count {
                    attributes.push(Self::read(src, const_items)?);
                }
                Ok(Self::Code {
                    max_stack,
                    max_locals,
                    ops: ops.into(),
                    attributes: attributes.into(),
                    exceptions: exceptions.into(),
                })
            }
            "ConstantValue"=>{
                let value_index = load_u16(src)?;
                 Ok(Self::ConstantValue{value_index})
            },
            "EnclosingMethod"=>{
                let class_index = load_u16(src)?;
                let method_index = load_u16(src)?;
                Ok(Self::EnclosingMethod{class_index,method_index})
            },
            "RuntimeVisibleParameterAnnotations"=>Ok(Self::Unknown), //TODO: Needed in the future.
            "RuntimeVisibleTypeAnnotations"=>Ok(Self::Unknown), //TODO: Needed in the future.
            "AnnotationDefault"=>Ok(Self::Unknown), //TODO: Needed in the future.
            "RuntimeInvisibleTypeAnnotations"=>Ok(Self::Unknown), //TODO: Not needed, but might be needed in the future.
            "RuntimeInvisibleParameterAnnotations"=>Ok(Self::Unknown), //TODO: Not needed, but might be needed in the future.
            "RuntimeInvisibleAnnotations"=>Ok(Self::Unknown), //TODO: Not needed, but might be needed in the future.
            "ST_TERMINATED" | "()Lio/netty/buffer/ByteBuf;" | "I" | "Index" | "Lcom/google/common/cache/ReferenceEntry<TK;TV;>;" | "Ljava/util/Comparator;" => 
            return Err(std::io::Error::new(std::io::ErrorKind::Other,format!("Nonsense attribute \"{attribute_name}\""))),//TODO: 100% result of an error parsing a class.
            _ => todo!("Can't read attributes of type {attribute_name}!"),
        }
    }
    pub(crate) fn read<R: std::io::Read>(
        src: &mut R,
        const_items: &[ConstantItem],
    ) -> Result<Self, std::io::Error> {
        let attribute_name_index = load_u16(src)?;
        //assert!(attribute_name_index > 0);
        if attribute_name_index == 0{
            return Err(std::io::Error::new(std::io::ErrorKind::Other,"AttributeNameIndex is 0!"));
        }
        let attribute_name = &const_items.get((attribute_name_index - 1) as usize).ok_or_else(||{std::io::Error::new(std::io::ErrorKind::Other,"AttributeNameIndex is outside ConstItem.")})?;
        let attribute_name = if let ConstantItem::Utf8(attribute_name) = attribute_name {
            attribute_name
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other,"Atribute name must be a UTF8 string!"));
        };
        let attribute_length = load_u32(src)? as usize;
        let mut attibute_data = vec![0; attribute_length];
        src.read(&mut attibute_data)?;
        Self::decode_attribute(&mut &attibute_data[..], attribute_name, const_items)
    }
}
