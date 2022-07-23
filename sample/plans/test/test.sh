#!/usr/bin/bash

ls /home && whoami | awk '{print "Hello " $1 }'
