use std::process::Command;
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
        if !message.contains("already exists and is not an empty directory"){
            println!("error:{}",message);
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
        if !message.contains("already exists and is not an empty directory"){
            println!("error:{}",message);
        }
    }
}
fn run_autogen(){
    let mut path =std::env::var_os("OUT_DIR").expect("OUT_DIR is not set").to_owned();
    //This is NOT how docs describe this ought to work!
    path.push("/bdwgc/");
    let result = Command::new("sh")
            .current_dir(path)
            .arg("./autogen.sh")
            .output()
            .expect("failed to execute process");
    if(result.stderr.len() > 0){
        println!("error:{}",std::str::from_utf8(&result.stderr).expect("Error message not UTF-8"));
    }
}
fn run_configure(){
    let mut path =std::env::var_os("OUT_DIR").expect("OUT_DIR is not set").to_owned();
    //This is NOT how docs describe this ought to work!
    path.push("/bdwgc/");;
    let result = Command::new("sh")
            .current_dir(path)
            .arg("./configure")
            .arg("--enable-cplusplus")
            .output()
            .expect("failed to execute process");
    if(result.stderr.len() > 0){
        println!("error:{}",std::str::from_utf8(&result.stderr).expect("Error message not UTF-8"));
    }
}
fn run_make(){
    let mut path =std::env::var_os("OUT_DIR").expect("OUT_DIR is not set").to_owned();
    //This is NOT how docs describe this ought to work!
    path.push("/bdwgc/");
    let result = Command::new("make")
            .current_dir(path)
            .arg("-j")
            .output()
            .expect("failed to execute process");
    if(result.stderr.len() > 0){
        println!("error:{}",std::str::from_utf8(&result.stderr).expect("Error message not UTF-8"));
    }
}
fn run_make_check(){
    let mut path =std::env::var_os("OUT_DIR").expect("OUT_DIR is not set").to_owned();
    //This is NOT how docs describe this ought to work!
    path.push("/bdwgc/");
    let result = Command::new("make")
            .current_dir(path)
            .arg("check")
            .output()
            .expect("failed to execute process");
    if(result.stderr.len() > 0){
        println!("error:{}",std::str::from_utf8(&result.stderr).expect("Error message not UTF-8"));
    }
}
fn main(){
    clone_bdwgc();
    clone_libatomic_ops();
    run_autogen();
    run_configure();
    run_make();
    run_make_check();
}
