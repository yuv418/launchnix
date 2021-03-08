let
  launchnixDeployment = import {{ deployment_abspath }}; # Will get replaced by rust
  pkgs = import <nixpkgs> {};
in
{

  network = {
    inherit pkgs;
    description = "Launchnix deployments";
  };

  "{{ dom_ip }}" = {config, pkgs, ...}: {
    imports = [
      launchnixDeployment.machine
      (import {{ hwconfig_path }})
    ];

    users.users.root.openssh.authorizedKeys.keys = [ (builtins.readFile "{{ sshpubkey_abspath }}") ];
    networking.hostName = launchnixDeployment.deployment.name;

    {% if static_ips %}
      networking.interfaces.ens3.useDHCP = true; # We do this so it's easier to choose a static ip
      networking.interfaces.ens3.ipv4.addresses = [
        {% for static_ip in static_ips %}
          {
            address = "{{ static_ip }}";
            prefixLength = 24; # TODO let people change this
          }
        {% endfor %}
      ];
    {% endif %}
  };

}
