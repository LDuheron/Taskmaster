Vagrant.configure("2") do |config|
  config.vm.box = "archlinux/archlinux"
  config.vm.provider "virtualbox" do |vb|
    vb.gui = false
    vb.cpus = 4
    vb.memory = "4096"
  end

  config.vm.provision "shell", inline: <<-SHELL
    pacman -Syu --noconfirm
    pacman -S --noconfirm rust fish
    chsh -s /bin/fish vagrant
  SHELL
end
