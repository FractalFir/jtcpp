use std::io::Result;
use std::process::{Child, Command};
fn clone_bdwgc(){
    let path = std::env::var_os("OUT_DIR").expect("OUT_DIR is not set").to_owned();
    let result = Command::new("git")
            .current_dir(path)
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/ivmai/bdwgc")
            .output()
            .expect("failed to execute process");
    if(result.stderr.len() > 0){
        let message = std::str::from_utf8(&result.stderr).expect("Error message not UTF-8");
        if !(message.contains("already exists and is not an empty directory") || message.contains("Cloning into '")){
            panic!("error:{}",message);
        }
    }
}
fn clone_libatomic_ops(){
    let mut path =std::env::var_os("OUT_DIR").expect("OUT_DIR is not set").to_owned();
    //This is NOT how docs describe this ought to work!
    path.push("/bdwgc/");
    let result = Command::new("git")
            .current_dir(path)
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/ivmai/libatomic_ops")
            .output()
            .expect("failed to execute process");
    if(result.stderr.len() > 0){
        let message = std::str::from_utf8(&result.stderr).expect("Error message not UTF-8");
        if !(message.contains("already exists and is not an empty directory") || message.contains("Cloning into '")){
            panic!("error:{}",message);
        }
    }
}
fn configure_cmake<P: AsRef<std::path::Path>>(compile_path: P) {
    let result = Command::new("cmake")
        .current_dir(compile_path)
        .arg("-Denable_cplusplus=ON")
        .arg("..")
        .output()
        .unwrap();
    if(result.stderr.len() > 0){
        let message = std::str::from_utf8(&result.stderr).expect("Error message not UTF-8");
        if !(message.contains("Explicit GC_INIT() calls may be required.")){
            panic!("error:{}",message);
        }
    }
}
fn cmake_build<P: AsRef<std::path::Path>>(compile_path: P) {
    let result =  Command::new("cmake")
        .current_dir(compile_path)
        .arg("--build")
        .arg(".")
        .output()
        .unwrap();
    if(result.stderr.len() > 0){
        let message = std::str::from_utf8(&result.stderr).expect("Error message not UTF-8");
        panic!("error:{}",message);
            
    }
}
fn main() {
    clone_bdwgc();
    clone_libatomic_ops();
    let mut compile_path = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR is not set")
        .to_owned();
    compile_path.push("/bdwgc/out");
    std::fs::create_dir_all(&compile_path).unwrap();
    configure_cmake(&compile_path);
    cmake_build(&compile_path);
}
