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

    networking.interfaces.ens3.useDHCP = true; # We do this so it's easier to choose a static ip
    {% if static_ips %}
        {% for static_ip in static_ips %}
        networking.interfaces.{{ static_ip.interface }}.ipv4.addresses = [
          {% for ip in static_ip.ips %}
          {
            address = "{{ ip }}";
            prefixLength = {{ static_ip.prefix }}; # TODO let people change this
          }
          {% endfor %}
        ];
        {% endfor %}
    {% endif %}
  };

}
