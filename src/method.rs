use crate::{method_desc_to_args, FatOp, IString, ImportedJavaClass, VariableType,fatops::ClassInfo};
pub(crate) struct Method {
    is_virtual: bool,
    class_name: IString,
    name: IString,
    ops: Box<[FatOp]>,
    args: Vec<VariableType>,
    ret_val: VariableType,
}
impl Method {
    pub(crate) fn is_virtual(&self) -> bool {
        self.is_virtual
    }
    pub(crate) fn class_name(&self) -> &str {
        &self.class_name
    }
    pub(crate) fn ret_val(&self) -> &VariableType {
        &self.ret_val
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn args(&self) -> &[VariableType] {
        &self.args
    }
    pub(crate) fn ops(&self) -> &[FatOp] {
        &self.ops
    }
    pub(crate) fn from_raw_method(
        method: &crate::importer::Method,
        name: &str,
        jc: &ImportedJavaClass,
    ) -> Method {
        let name: IString = name.into();
        let (mut args, ret_val) = method_desc_to_args(method.descriptor(jc));
        let is_virtual = method.is_virtual(jc);
        let ops = match method.bytecode() {
            Some(ops) => crate::fatops::expand_ops(ops, jc),
            None => [].into(),
        };
        if name.contains("_init_"){
            args.insert(0,VariableType::ObjectRef(ClassInfo::from_java_path(jc.name())))
        }
        Method {
            class_name: jc.lookup_class(jc.this_class()).unwrap().into(),
            is_virtual,
            name,
            args,
            ret_val,
            ops,
        }
    }
}
