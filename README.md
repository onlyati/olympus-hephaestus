# Hephaestus@Olympus

## :earth_africa: What is Olympus?

Olympus is name of my package which is intended to supervise a Linux server environment and provide its applications and services a stable backend. Olympus consist of:
- **Zeus:** Responsible to run defined applications on proper server machines
- **Hermes:** Act like an in-memory database and a message queue for other Olympus applications
- **Chronos:** Execute commands by timely manner
- **Hephaestus:** Run long and complex tasks in the background as jobs
- **Apollo:** Center of documentation, stores every information and thresholds for monitoring scripts
- **Argos:** Collecting and analyzing data and forward it to Athena
- **Athena:** Automation of Olympus, it analyzes what other component does and act according to its rules

## :hammer: Structure of Hephaestus server

Hephaestus is a single procedure and compiled into a single binary file. After startup, it establish a gRPC server using [hephaestus.proto](hephaestus/proto/hephaestus.proto) file.
Depends from configuration, it listen for secured or non-secured connections.

Hephaestus stores its plan in simple files. Location of this directory is specified in configuration file. Each plan consist of steps. Steps might depend from each other but they are executed after each other.

Hephaestus also has a CLI program too, by this communication can be done from command line too.
Following actions can be done from this interface:
```
Usage: cli [OPTIONS] --hostname <HOSTNAME> <COMMAND>

Commands:
  list-plan-sets    List all plan set
  list-plans        List all plan within a set
  list-plan         Get details about specified plan
  plans             List the scheduled plan output from memory
  status            Show status and log of a scheduled plan
  exec              Execute a specified plan
  dump-history      Write a specific scheduled plan output into file
  dump-all-history  Write all scheduled plan output into files
  help              Print this message or the help of the given subcommand(s)

Options:
  -H, --hostname <HOSTNAME>  Where it should connect
                             Allowed formats:
                             - <protocol>://<hostname>:<port>, for example http://127.0.0.1:3041
                             - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate
  -c, --config <CONFIG>      Config file for connection details [default: /etc/olympus/hephaestus/client.conf]
  -v, --verbose              Show more detail about connection
  -h, --help                 Print help information
```


