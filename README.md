# TASKMASTER

The project taskmaster from 42Networks aims to make a fully fledged job control daemon.

"Your job here is to make a fully-fledged job control daemon. A pretty good example of this would be supervisor. For the sake of keeping it simple, your program will not run as root, and does not HAVE to be a daemon. It will be started via shell, and do its job while providing a control shell to the user".

-> programme conçu pour superviser et contrôler d'autres programmes (ou processus) sur un système. Il assure le bon fonctionnement de ces programmes en les redémarrant automatiquement en cas de panne

daemon -> programme en arriere plan

Avoir deux "programmes" ? un pour le daemon et un pour linterface de communication

## Configuration fields

- command
- numprocs
- autostart
- autorestart
- exitcodes
- startsecs
- startretries
- stopsignals
- stopwaitsecs
- stdout
- stderr
- workdir
- environment
- umask
