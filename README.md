TASKMASTER

The project taskmaster from 42Networks aims to make a fully fledged job control daemon.

"Your job here is to make a fully-fledged job control daemon. A pretty good example of this would be supervisor. For the sake of keeping it simple, your program will not run as root, and does not HAVE to be a daemon. It will be started via shell, and do its job while providing a control shell to the user".

-> programme conçu pour superviser et contrôler d'autres programmes (ou processus) sur un système. Il assure le bon fonctionnement de ces programmes en les redémarrant automatiquement en cas de panne 

daemon -> programme en arriere plan



Avoir deux "programmes" ? un pour le daemon et un pour linterface de communication 


tkmaster -> gestionnaire du service. 

launchd, init, upervisor

difference supervisor et taskmaster -> ne doit pas se substituer a init et etre le pid un systeme unix.
rogramme qui peut se lancer en user space et gerer des programmes elon un ficher de config-> lancer es programmes, les maintenir en vie (crash, kill..), les relancer, relancer que 'il y a un certain code d'erreur, que un certain nombre de fois ou a linfini, si on veut rediriger la sortie standard, d'erreur, yaml pour le fichier de config.

systeme de login

le programme doit rester au premier plan 

pouvoir changer le config file sans arreter le main program

