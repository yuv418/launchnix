#![recursion_limit="256"] mod xml;
mod nix;
mod vm;
mod image;
mod morph;

fn main() {
    let mut vm = vm::VM::from_nixfile("./examples/example-deployment.nix");
    vm.apply();
}
