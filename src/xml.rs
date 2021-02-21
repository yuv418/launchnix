use amxml::dom::*;

pub fn merge_xml(original_xml: &str, to_merge_xml: &str, root_attr: &str) -> String {
    let mut doc = new_document(&original_xml).unwrap();
    let merge_doc = new_document(&to_merge_xml).unwrap();

    _merge(&mut doc, merge_doc.root_element(), &format!("//{}", root_attr));

    doc.root_element().to_string()
}

fn _merge(mut original: &mut NodePtr, new: NodePtr, parent_xpath: &str) {
    // println!("document now is: {:?}", new.children());
    // println!("document orig is: {}", original.to_string());


    for child in new.children() { // TODO attributes

        if &child.name() != "" {

            let child_xpath = parent_xpath.to_string() + "/" + &child.name();

            // println!("child xpath {:#?}", child_xpath);
            // println!("child children {:?}", child.children()[0].children());

            if let None = original.get_first_node(&child_xpath) {
                if let Some(parent_original_node) = original.get_first_node(&parent_xpath) { // You could be creating a "sub-node"
                    parent_original_node.append_child(&child); // Push child node into the parent node if the parent doesn't already have a child of the same name.
                }
            }
            else if child.children().len() == 1 {
                // println!("Yes, we are here.");
                if child.children()[0].children().len() == 0 { // Child has no children, "data" node of sorts
                    if let Some(replace_node) = original.get_first_node(&child_xpath) {
                        //println!("child xpath finder {:#?}", original.get_first_node(&child_xpath));
                        // println!("Here");

                        // Disallow overwriting blank nodes, since that would be silly

                        if child.children()[0].inner_xml() != "" {
                            replace_node.replace_with(&child); // Replace original node with this node
                        }
                    }
                }
            }
            else { // Child has children
                if let Some(mut copy_node) = original.get_first_node(&child_xpath) { // Copy attributes of child that exists in parent (we would be merging child's children)
                    for attr in child.attributes() {
                        // println!("Copying attr {}", attr.name());
                        copy_node.set_attribute(&attr.name(), &attr.value());
                    }
                }
                _merge(&mut original, child, &child_xpath);
            }
        }

    }

}
