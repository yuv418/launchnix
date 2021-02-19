{
    deployment = let
        a = 32;
        b = 1;
    in
    {
        cpus = b + 3;
        ram = 4096;
        extraConfig = ''
<devices>
        <driver name='qemu' type='raw' cache='none' io='native' discard='unmap' iothread='1' queues='8'/>
    <source file='/var/lib/libvirt/images/pool/win10.img'/>
    <target dev='vda' bus='virtio'/>
</devices>
'';
        name = "test";
    };
foo = {
  baz = ''
bar
more things here
'';
};
    machine =

    {config, pkgs, ...}:

    {

    };

}
