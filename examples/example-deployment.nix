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
        diskSize = 4096;

        name = "test";
        sshPubKeyPath = "~/.ssh/id_rsa.pub";
        sshPrivKeyPath = "~/.ssh/id_rsa";
        staticIPs = [
            {
                ips = [ "192.168.122.32" "192.168.122.48" ];
            }
        ];

        installationMedia = builtins.fetchurl "https://dl-cdn.alpinelinux.org/alpine/v3.13/releases/x86_64/alpine-standard-3.13.5-x86_64.iso";

        # TODO make launchnix complain if you try to convert a non-NixOS domain into a NixOS domain (this operation will be unsupported).
    };

    machine = {config, pkgs, lib, ...}:
    {
        services.traefik.enable = true;
    };
}
