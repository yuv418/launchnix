use serde::Deserialize;
use format_xml::xml;
use amxml::dom::*;
use virt::connect::Connect;
use virt::storage_pool::StoragePool;
use virt::domain::{Domain, VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE};
use virt::error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::fs::Permissions;
use std::{thread, time};
use std::net::TcpStream;
use ssh2::Session;
use std::io::prelude::*;


use crate::nix;
use crate::image;
use crate::morph;

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VM {
    extra_config: String,
    cpus: i32,
    ram: i32,
    name: String,
    ssh_pub_key_path: String,
    ssh_priv_key_path: String,

    #[serde(skip)]
    file_path: String
}

impl VM {
    pub fn from_nixfile(file_path: &str) -> Self {
        let mut vmparams: Self = nix::from_nixfile(file_path);
        vmparams.file_path = file_path.to_string();

        vmparams.extra_config = format!("<domain>{}</domain>", vmparams.extra_config.replace("\\\\", "\\")); // Wrap config in domain tags and unescape \ns (because nix eval does weird things)
        vmparams
    }

    pub fn apply(&self) -> Result<(), error::Error>{
        let mut conn = Connect::open("qemu:///system")?;

        if let Err(error::Error { .. }) = Domain::lookup_by_name(&conn, &self.name) { // Only create a new domain if it doesn't exist.
            // Create the VM since it doesn't exist
            let oqcow2 = image::build_image(&self.ssh_pub_key_path).unwrap();
            // let oqcow2 = oqcow2_path.to_str().unwrap(); // TODO Copy image to libvirt storage
            // let oqcow2 = "nix/result/nix.qcow2";
            let defspool = StoragePool::lookup_by_name(&conn, "default")?;
            let defspool = new_document(&defspool.get_xml_desc(virt::storage_pool::STORAGE_POOL_CREATE_NORMAL)?).unwrap(); // Load into amxml and then get xpath /pool/target/path
            println!("defspool: {}", defspool.to_string());

            if let Some(defspool) = defspool.get_first_node("//pool/target/path") {
                let copy_path = defspool.inner_xml() + &format!("/{}.qcow2", self.name);

                println!("Copied {} to {}", oqcow2, copy_path);
                fs::copy(&oqcow2, &copy_path).unwrap();

                let perms = Permissions::from_mode(0o700);
                println!("perms mode {}", perms.mode());
                fs::set_permissions(&copy_path, perms).unwrap();

                // TODO UUID must be set to patch an existing domain
                let base_xml = xml! {
                    <domain type="kvm">
                    <name>{self.name}</name>
                    <memory unit="MiB">{self.ram}</memory>
                    <vcpu>{self.cpus}</vcpu>
                    <uuid></uuid>
                    <os>
                        <type arch="x86_64" machine="pc-i440fx-5.1">"hvm"</type>
                    </os>
                    <devices>
                        <disk type="file" device="disk">
                            <driver name="qemu" type="qcow2" />
                            <source file={copy_path} />
                            <target dev="hda" bus="ide" />
                            <boot order="1" />
                        </disk>
                        <interface type="network">
                            <source network="default"/>
                            <model type="rtl8139"/>
                        </interface>
                        <graphics type="spice"/>
                        <video>
                            <model type="qxl" />
                        </video>
                        <input type="keyboard" bus="usb" />
                    </devices>
                    </domain>
                }.to_string();

                println!("{}", base_xml);

                let dom = Domain::define_xml(&conn, &base_xml)?;
                dom.create();

                println!("Waiting for domain IP...");

                let mut iaddrs = dom.interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;
                while iaddrs.len() == 0 {
                    thread::sleep(time::Duration::from_secs(3)); // TODO maybe change this
                    iaddrs = dom.interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;
                }

                let sship = &iaddrs[0].addrs[0].addr;
                println!("Found IP: {}", sship);


                println!("{:?}", morph::exec_morph(&sship, &self.ssh_pub_key_path, &self.file_path));
                // Create SSH session

                /*let mut sshsess = Session::new().unwrap();

                sshsess.set_tcp_stream(TcpStream::connect(sship.to_owned() + ":22").unwrap());
                sshsess.handshake().unwrap();
                sshsess.userauth_agent("root").unwrap(); // TODO make this a setting (use password/agent?)

                let mut channel = sshsess.channel_session().unwrap();
                channel.exec("ls /").unwrap();
                let mut s = String::new();
                channel.read_to_string(&mut s).unwrap();
                println!("{}", s);
                channel.wait_close();
                println!("{}", channel.exit_status().unwrap());*/

            }



        }

        Ok(())
    }


}
