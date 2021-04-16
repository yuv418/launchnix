{
    deployment =
    {
        cpus = 4;
        ram = 2048;
        extraConfig = ''
        <devices>
            <serial type="pty"/>
        </devices>
        '';

        # diskSize = 4096; # replace all instances of disk_size with initialDisks.boot.size

        name = "test";
        sshPubKeyPath = "~/.ssh/id_rsa.pub";
        sshPrivKeyPath = "~/.ssh/id_rsa";
        staticIPs = [
            {
                ips = [ "192.168.122.32" "192.168.122.48" ];
            }
        ];

        installationMedia = builtins.fetchurl "https://dl-cdn.alpinelinux.org/alpine/v3.13/releases/x86_64/alpine-standard-3.13.5-x86_64.iso";
        initialDisks = {
            boot = { # RESERVED name
                size = 8192; # this will change if the backup file size is different. TODO perform resize automagically?
                fromBackup = "/home/nonuser/Documents/test1.qcow2";
            };
        };

        # TODO make launchnix complain if you try to convert a non-NixOS domain into a NixOS domain (this operation will be unsupported).
    };

    machine = {config, pkgs, lib, ...}:
    {
        services.traefik.enable = true;
    };
}
