Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/jammy64"
  config.vm.provider "virtualbox" do |vb|
    vb.gui = false
    vb.cpus = 8
    vb.memory = "8096"
  end

  config.vm.provision "shell", inline: <<-SHELL
    apt update
    apt install -y cargo fish
    chsh -s /bin/fish vagrant
  SHELL
end
