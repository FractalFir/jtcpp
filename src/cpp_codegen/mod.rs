use crate::{Class,IString};
use std::io::Write;
use std::collections::HashSet;
struct HeaderBuilder{
    header:String,
    includes:HashSet<IString>,
}
impl HeaderBuilder{
    fn new(start:&str) -> Self{
        Self{header:start.into(),includes:HashSet::new()}
    }
    fn add_include(&mut self,include:&str){
        if let None = self.header.get(include){
            self.header.push_str(&format!("#include \"{include}.h\"\n"));
            self.includes.insert(include.into());
        }
    }
}
fn push_method_sig(target:&mut String,method:crate::Method){

}
fn create_header<W:Write>(out:&mut W, class:&Class)->std::io::Result<()>{
    let mut header = HeaderBuilder::new("#pragma once\n");
    header.add_include(class.parrent_name());
    let mut class_methods = String::new();
    for method in class.static_methods(){

    }
    todo!();
    //let class_body
}
