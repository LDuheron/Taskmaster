# TASKMASTER

<!--toc:start-->
- [Configuration](#configuration)
<!--toc:end-->

The project taskmaster from 42Networks aims to make a fully fledged job control daemon.

"Your job here is to make a fully-fledged job control daemon. A pretty good example of this would be supervisor. For the sake of keeping it simple, your program will not run as root, and does not HAVE to be a daemon. It will be started via shell, and do its job while providing a control shell to the user".

-> programme conçu pour superviser et contrôler d'autres programmes (ou processus) sur un système. Il assure le bon fonctionnement de ces programmes en les redémarrant automatiquement en cas de panne

daemon -> programme en arriere plan

Avoir deux "programmes" ? un pour le daemon et un pour linterface de communication

## Configuration

- command: `command [arguments]`
- numprocs: `numeric`
- autostart: `true | false`
- autorestart: `never | unexpected | always`
- exitcodes: `numeric[, numeric...]`
- startsecs: `numeric`
- startretries: `numeric`
- stopsignal: `hup | int | quit | kill | usr1 | usr2 | term` (not case sensitive)
- stopwaitsecs: `numeric`
- stdout: `filename`
- stderr: `filename`
- workdir: `filename`
- environment: `key=value[, key=value...]`
- umask: `033` (octal value for umask)
