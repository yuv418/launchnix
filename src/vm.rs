use serde::Deserialize;
use format_xml::xml;
use virt::connect::Connect;
use virt::storage_pool::StoragePool;
use virt::domain::Domain;
use virt::error;

use crate::nix;
use crate::image;

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VM {
    extra_config: String,
    cpus: i32,
    ram: i32,
    name: String,
}

impl VM {
    pub fn from_nixfile(file_path: &str) -> Self {
        let mut vmparams: Self = nix::from_nixfile(file_path);

        vmparams.extra_config = format!("<domain>{}</domain>", vmparams.extra_config.replace("\\\\", "\\")); // Wrap config in domain tags and unescape \ns (because nix eval does weird things)
        vmparams
    }

    pub fn apply(&self) -> Result<(), error::Error>{
        let mut conn = Connect::open("qemu:///system")?;

        if let Err(error::Error { .. }) = Domain::lookup_by_name(&conn, &self.name) { // Only create a new domain if it doesn't exist.
            // Create the VM since it doesn't exist
            // let oqcow2 = image::build_image(); // TODO Copy image to libvirt storage
            let oqcow2 = "nix/result/nix.qcow2";
            let defspool = StoragePool::lookup_by_name(&conn, "default")?;
            println!("{}", defspool.get_xml_desc(virt::storage_pool::STORAGE_POOL_CREATE_NORMAL)?); // Load into amxml and then get xpath /pool/target/path

            // TODO UUID must be set to patch an existing domain

            let base_xml = xml! {

<domain type="kvm">
<name>{self.name}</name>
<memory>{self.ram}</memory>
<vcpu>{self.cpus}</vcpu>
<uuid></uuid>
<os>
    <type arch="x86_64" machine="pc-i440fx-5.1">"hvm"</type>
    <boot dev="hd"/>
</os>
<devices>
    <disk type="file" device="disk">
        <source file={oqcow2} />
        <target dev="hda" />
    </disk>
    <interface type="network">
        <source network="default"/>
        <model type="rtl8139"/>
    </interface>
</devices>
</domain>
            }.to_string();
            println!("{}", base_xml);

            Domain::define_xml(&conn, &base_xml)?;
        }

        Ok(())
    }

}
