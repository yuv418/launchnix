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

        name = "test";
        sshPubKeyPath = "~/.ssh/id_rsa.pub";
        sshPrivKeyPath = "~/.ssh/id_rsa";
    };

    machine = {config, pkgs, lib, ...}:
    {
        services.nginx.enable = true;
        services.k3s.enable = true;
    };
}
