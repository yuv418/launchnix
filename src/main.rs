#![recursion_limit="256"] mod xml;
mod nix;
mod vm;
mod image;
mod morph;

fn main() {
    let o_xml = r#"
<a>
  <b id="2">c</b>
  <e>fg</e>
<m><kh>z</kh></m>
</a>
"#;
    let to_merge_xml = r#"
<a hi="yes">
  <b id="1">o</b>
  <m x="quux">
   <q>r</q>
  <quuux />
</m>
  <c>d</c>
</a>
"#;



     // println!("document now is: {}", xml::merge_xml(o_xml, to_merge_xml, "a"));

    let vm = vm::VM::from_nixfile("./examples/example-deployment.nix");
    vm.apply();



    // println!("{}", );


}

