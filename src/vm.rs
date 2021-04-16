use amxml::dom::*;
use format_xml::xml;
use serde::{Deserialize, Serialize};

use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::{
    collections::hash_map::DefaultHasher,
    collections::BTreeMap,
    fs,
    fs::Permissions,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use virt::{
    connect::Connect,
    domain::{Domain, VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, VIR_DOMAIN_NONE},
    error,
    storage_pool::StoragePool,
};

use crate::morph;
use crate::nix;
use crate::nix_image;
use crate::qcow2;
use crate::xml::merge_xml;

#[derive(Default, Debug, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct VM {
    extra_config: String,
    cpus: u32,
    ram: u32,
    name: String,
    ssh_pub_key_path: String,
    ssh_priv_key_path: String,

    #[serde(rename = "staticIPs")] // Serde won't rename this one for me, unfortunately
    static_ips: Option<Vec<StaticIP>>, // Each entry will be for a different interface

    #[serde(default = "default_storage_pool_name")]
    storage_pool_name: String,

    #[serde(default = "default_disk_size")]
    disk_size: u64, // In MiB, or something TODO replace with initial_disks
    #[serde(skip)]
    file_path: String,

    #[serde(skip)]
    image_path: String,

    // Unimplemented options (including non-NixOS options)
    vgpu_type: Option<String>, // for NVIDIA vGPU support (eg. "nvidia-62"). Will automatically declaratively create/update/set the mdev on change/create
    pci_devices: Option<Vec<PCIDevice>>, // for PCI passthrough.
    initial_disks: Option<BTreeMap<String, InitialDiskConfiguration>>,

    // Non-NixOS options only—will not run morph if any of these are defined
    nixos: Option<bool>, // if this isn't explicitly set -> if installation_media.is_some() || boot_order.is_some() { false } else { true }
    installation_media: Option<PathBuf>, // Path to installation media
    boot_order: Option<Vec<String>>, // Disk name list?
}

#[derive(Deserialize, Serialize, Debug, Hash)]
pub struct InitialDiskConfiguration {
    size: u64,
    from_backup: Option<PathBuf>, // should this be PathBuf?
    extra_xml: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, Hash)]
pub struct PCIDevice {
    id: String,
    gpu: bool,
    rom_file: Option<String>,
    extra_xml: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Hash)]
pub struct StaticIP {
    ips: Vec<String>,

    #[serde(default = "ens3")]
    interface: String,

    #[serde(default = "get24")]
    prefix: u32,
}

fn default_disk_size() -> u64 {
    8192
}

fn get24() -> u32 {
    24
}

fn ens3() -> String {
    "ens3".to_owned()
}

fn default_storage_pool_name() -> String {
    "default".to_owned()
}

impl VM {
    // Helper function because of how the `nixos` flag is handled
    pub fn nixos(&self) -> bool {
        if self.nixos.is_some() {
            return self.nixos.unwrap();
        }
        true
    }
    pub fn from_nixfile(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut vmparams: Self = nix::from_nixfile(file_path)?;
        vmparams.file_path = file_path.to_string();

        vmparams.extra_config = format!(
            "<domain type=\"kvm\">{}</domain>",
            vmparams.extra_config.replace("\\\\", "\\")
        ); // Wrap config in domain tags and unescape \ns (because nix eval does weird things)

        // Determine whether this is a NixOS deployment or a non-NixOS deployment
        // Don't do any determination if the the user sets the flag manually
        if vmparams.nixos.is_none() {
            vmparams.nixos =
                Some(vmparams.installation_media.is_none() && vmparams.boot_order.is_none());
        }

        /*
        TODO add validation method (will be called right here)

        installationMedia existing
        vGPUType being a valid MDEV type
        pciDevices containing valid PCI devices/ROM files
        initialDisks's diskSize < storageSize and backupFile existing if specified

        */
        Ok(vmparams)
    }

    fn hash_self(&self) -> u64 {
        // borrowed from https://doc.rust-lang.org/std/hash/index.html
        let mut defhasher = DefaultHasher::new();
        self.hash(&mut defhasher);
        defhasher.finish()
    }

    fn copy_build_image(
        &mut self,
        conn: &Connect,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.image_path = self.deployment_image_path(conn)?;

        if self.nixos() {
            let oqcow2 = nix_image::build_image(&self.ssh_pub_key_path, self.disk_size).unwrap();

            println!("DEBUG STORAGE: Copying {} to {}", oqcow2, self.image_path);
            fs::copy(&oqcow2, &self.image_path)?;
        } else {
            // TODO unnecessary allocation, deployment_image_path should return PathBuf, not String
            let image_pathbuf = PathBuf::from(&self.image_path);
            qcow2::create(&image_pathbuf, self.disk_size)?;
        }

        let perms = Permissions::from_mode(0o700);
        fs::set_permissions(&self.image_path, perms)?;

        Ok(())
    }

    fn deployment_image_path(&self, conn: &Connect) -> Result<String, Box<std::error::Error>> {
        let spool = StoragePool::lookup_by_name(conn, &self.storage_pool_name)?;
        let spool =
            new_document(&spool.get_xml_desc(virt::storage_pool::STORAGE_POOL_CREATE_NORMAL)?)
                .unwrap(); // Load into amxml and then get xpath /pool/target/path

        if let Some(spool) = spool.get_first_node("//pool/target/path") {
            return Ok(spool.inner_xml() + &format!("/{}.qcow2", self.name));
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get storage pool path",
        )))
    }

    fn dom_uuid(&self, dom: &Domain) -> Result<String, Box<std::error::Error>> {
        let domxml = new_document(&dom.get_xml_desc(VIR_DOMAIN_NONE)?)?;
        if let Some(uuid) = domxml.get_first_node("//domain/uuid") {
            return Ok(uuid.inner_xml());
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get UUID",
        )))
    }

    fn dom_deployment_hash(&self, dom: &Domain) -> Result<u64, Box<std::error::Error>> {
        // :( repeated code
        let domxml = new_document(&dom.get_xml_desc(VIR_DOMAIN_NONE)?)?;
        if let Some(uuid) = domxml.get_first_node("//domain/metadata/launchnix:deploymentXMLHash") {
            return Ok(uuid.inner_xml().parse()?); // Bad
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get deployment hash",
        )))
    }

    pub fn reboot(&self, dom: &Domain) -> Result<String, Box<dyn std::error::Error>> {
        /*
        Reboot the domain, and if the domain is not started, start it up.
        Return the IP of the domain when it's finished coming up.
        */
        self.shutdown(dom)?;

        // Wait till it has an IP, which will auto-start the VM

        self.dom_ip(dom)
    }

    pub fn shutdown(&self, dom: &Domain) -> Result<(), Box<dyn std::error::Error>> {
        dom.shutdown()?;
        // TODO we need something here to make sure we don't have an infinite loop
        while dom.is_active()? {
            // thread::sleep(time::Duration::from_secs(2)); // TODO maybe change this
        }

        Ok(())
    }

    pub fn apply_xml(
        &self,
        conn: &Connect,
        domopt: Option<Domain>,
    ) -> Result<Domain, Box<dyn std::error::Error>> {
        // TODO implement for running domains
        // UUID must be set to patch an existing domain

        let base_xml = xml! {
            <domain type="kvm">
            <name>{self.name}</name>
            if let Some(dom) = (&domopt) {
                <uuid>{self.dom_uuid(&dom).unwrap()}</uuid>
            }
            <metadata>
                <launchnix:deploymentXMLHash xmlns:launchnix="/launchnix">{self.hash_self()}</launchnix:deploymentXMLHash>
            </metadata>
            <memory unit="MiB">{self.ram}</memory>
            <vcpu>{self.cpus}</vcpu>
            <uuid></uuid>
            <os>
                <type arch="x86_64" machine="pc-i440fx-5.1">"hvm"</type>
            </os>
            <devices>
                if (!self.nixos()) {
                    if let Some(installation_media) = (&self.installation_media) { // Boot up with install media (first)
                        <disk type="file" device="cdrom">
                            <driver name="qemu" type="raw"/>
                            <source file={installation_media.to_str().unwrap()}/>
                            <target dev="hdb" bus="ide"/>
                            <boot order="1"/>
                        </disk>
                    }
                }
                <disk type="file" device="disk">
                    <driver name="qemu" type="qcow2" />
                        <source file={self.image_path} />
                    <target dev="hda" bus="ide" />
                    <boot order="2" />
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
            <features>
                <acpi/>
            </features>
            </domain>
        }.to_string();

        let merged_xml = merge_xml(&base_xml, &self.extra_config, "domain");
        // dbg!(&merged_xml);

        // Dom exists, shut it down
        if let Some(dom) = &domopt {
            println!("DEBUG: Shutting down domain");
            self.shutdown(&dom); // We don't care if this fails.
            println!("DEBUG: Applying XML..."); // Haha we're not
        }

        println!("DEBUG: Starting up domain");
        let dom = Domain::define_xml(&conn, &merged_xml)?;
        dom.create()?;

        Ok(dom)
    }

    pub fn dom_ip(&self, dom: &Domain) -> Result<String, Box<std::error::Error>> {
        // Blocking function to receive domain IP

        // Start up the VM if it doesn't exist.
        if !dom.is_active()? {
            dom.create();
        }
        let mut iaddrs = dom.interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;
        while iaddrs.len() == 0 {
            // thread::sleep(time::Duration::from_secs(3)); // TODO maybe change this
            iaddrs = dom.interface_addresses(VIR_DOMAIN_INTERFACE_ADDRESSES_SRC_LEASE, 0)?;
        }

        Ok(iaddrs[0].addrs[0].addr.clone()) // First IP, TODO add more control here?
    }

    pub fn ssh(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Spawn an SSH subprocess with the dom's ip

        /* This will only get called from the CLI, so we can auto-connect here */

        let dom = self.dom(&self.conn())?;

        let mut ssh_exec = Command::new("ssh")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-i")
            .arg(&self.ssh_priv_key_path)
            .arg(format!("root@{}", self.dom_ip(&dom)?))
            .spawn()
            .expect("Launchnix failed to SSH into your deployment. Please report this issue.");

        ssh_exec.wait()?;

        Ok(())
    }

    pub fn conn(&self) -> Connect {
        Connect::open("qemu:///system") // TODO allow user-specified connections
            .expect("Couldn't conect to the supplied libvirt connection.")
    }

    pub fn dom(&self, conn: &Connect) -> Result<Domain, virt::error::Error> {
        Domain::lookup_by_name(conn, &self.name)
    }

    pub fn apply(&mut self) -> Result<(), Box<std::error::Error + 'static>> {
        // TODO use other connections if specified by the user
        let mut sship = String::new();
        let conn = self.conn();
        let domlookup = self.dom(&conn);
        let mut dom = None;

        if let Err(error::Error { .. }) = domlookup {
            // Only create a new domain if it doesn't exist.
            self.copy_build_image(&conn)?; // Don't build a NixOS if we're not building a NixOS guest
            dom = Some(self.apply_xml(&conn, None)?);
        } else if let Ok(mut dom_unw) = domlookup {
            // Domain already exists, apply XML and morph.
            self.image_path = self.deployment_image_path(&conn)?;
            // Check against libvirt metadata for any hash changes and apply changes if necessary
            if self.dom_deployment_hash(&dom_unw)? != self.hash_self() {
                println!("Configuration change detected. Applying VM changes...");
                dom = Some(self.apply_xml(&conn, Some(dom_unw))?);
            } else {
                dom = Some(dom_unw); // No changes, just assign the dom
            }
        }

        // Execute morph here
        // Overwrite SSH IP if the user set a static IP

        if self.nixos() {
            println!("DEBUG: Waiting for IP...");
            sship = self.dom_ip(&dom.unwrap())?;

            println!("DEBUG: Starting morph...");
            morph::exec_morph(
                &sship,
                &self.ssh_pub_key_path,
                &self.file_path,
                &self.static_ips,
            )?;
        } else {
            // nothing.
            println!("INFO: Non-NixOS VM; doing nothing.")
        }

        Ok(())
    }
}
