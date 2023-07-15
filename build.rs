use std::io::Result;
use std::process::{Child, Command};
fn handle_command(command: Result<Child>) {
    let result = command.expect("failed to execute process").wait().unwrap();
    let code = result.code().expect("command termianted by signal!");
    assert!(code == 0 || code == 128);
}
fn clone_bdwgc() -> Result<Child> {
    let path = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR is not set")
        .to_owned();
    Command::new("git")
        .current_dir(path)
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("https://github.com/ivmai/bdwgc")
        .spawn()
}
fn clone_libatomic_ops() -> Result<Child> {
    let mut path = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR is not set")
        .to_owned();
    //This is NOT how docs describe this ought to work!
    path.push("/bdwgc/");
    Command::new("git")
        .current_dir(path)
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("https://github.com/ivmai/libatomic_ops")
        .spawn()
}
fn configure_cmake<P: AsRef<std::path::Path>>(compile_path: P) -> Result<Child> {
    Command::new("cmake")
        .current_dir(compile_path)
        .arg("-Denable_cplusplus=ON")
        .arg("..")
        .spawn()
}
fn cmake_build<P: AsRef<std::path::Path>>(compile_path: P) -> Result<Child> {
    Command::new("cmake")
        .current_dir(compile_path)
        .arg("--build")
        .arg(".")
        .spawn()
}
fn main() {
    let mut commands = Vec::with_capacity(10);
    commands.push(clone_bdwgc());
    commands.push(clone_libatomic_ops());
    let mut compile_path = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR is not set")
        .to_owned();
    compile_path.push("/bdwgc/out");
    std::fs::create_dir_all(&compile_path).unwrap();
    commands.push(configure_cmake(&compile_path));
    commands.push(cmake_build(&compile_path));
    commands
        .into_iter()
        .for_each(|command| handle_command(command));
}
