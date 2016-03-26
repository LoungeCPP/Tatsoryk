# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
    config.vm.box = 'CatPlusPlus/Debian'
    config.vm.box_version = '~> 1.0.1'

    config.vm.network 'forwarded_port', guest: 80, host: 8000
    config.vm.network 'forwarded_port', guest: 443, host: 8443

    config.vm.provider 'virtualbox' do |vb|
        vb.cpus = 2
        vb.memory = '1024'
    end

    config.vm.provision 'shell', path: 'bin/provision'
end
