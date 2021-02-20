let
  launchnixDeployment = import deploymentPath; # Will get replaced by rust
  pkgs = import <nixpkgs> {};
in
{

  network = {
    inherit pkgs;
    description = "Launchnix deployments";
  };

  "domIP" = {config, pkgs, ...}: {
    imports = [
      launchnixDeployment.machine
      (import hwConfigPath)
    ];

    users.users.root.openssh.authorizedKeys.keys = [ (builtins.readFile "sshPubKeyPath") ];
    networking.hostName = launchnixDeployment.deployment.name;
  };

}
