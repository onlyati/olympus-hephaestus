# Client usage

The `--help` option of client describes what you can do:
```
Usage: hephaestus-cli [OPTIONS] --hostname <HOSTNAME> <ACTION>

Arguments:
  <ACTION>
          Specify the action what to do

          Possible values:
          - list-plan-sets:   List all plan set
          - list-plans:       List all plan within a set
          - list-plan:        Get details about specified plan
          - plans:            List the scheduled plan output from memory
          - status:           Show status and log of a scheduled plan
          - exec:             Execute a specified plan
          - dump-history:     Write a specific scheduled plan output into file
          - dump-all-history: Write all scheduled plan output into files

Options:
      --plan-name <PLAN_NAME>
          Specified plan's name

      --plan-set <PLAN_SET>
          Specified plen set's name

      --plan-id <PLAN_ID>
          Scheduled plan id

  -v, --verbose
          Display more information for debugging purpose

  -H, --hostname <HOSTNAME>
          Where it should connect
          Allowed formats:
          - <protocol>://<hostname>:<port>, for example http://127.0.0.1:3041
          - cfg://<definition-name>, for example: cfg://atihome, it will search  or hostname and CA certificate

  -c, --config <CONFIG>
          Config file for connection details

  -h, --help
          Print help information (use `-h` for a summary)
```

If secured connection is used, then connection information must be provided by sugin config file. This can be define with th `-c` option. If not defined, default is trying to be used: `/etc/olympus/hephaestus/client.conf`.
This config file can contain more server's host information. For example:
```
node.server1.address = https://server1.lan:9150
node.server1.ca_cert = /etc/olympus/hephaestus/certs/hepha_pr_ca.pem
node.server1.domain = server1.lan

node.server2.address = https://server1.lan:9150
node.server2.ca_cert = /etc/olympus/hephaestus/certs/hepha_pr_ca.pem
node.server2.domain = server1.lan
```

when `-H cfg://server1` or `-H cfg://server2` option is used, then connection information will be read from here.