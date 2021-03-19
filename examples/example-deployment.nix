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
    };

    machine = {config, pkgs, lib, ...}:
    {
        services.nginx.enable = true;
    };
}
