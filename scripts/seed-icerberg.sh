#!/bin/bash

ICEBERG_FOLDER="$HOME/projects/projet-reseau/torrent-iceberg"
OPENTRACKER_FOLDER="$HOME/projects/projet-reseau/opentracker"
OPENTRACKER_IP_ADDRESS="127.0.0.1"
OPENTRACKER_PORT=6969

$OPENTRACKER_FOLDER/opentracker -i $OPENTRACKER_IP_ADDRESS -p $OPENTRACKER_PORT & \
aria2c -V -d $ICEBERG_FOLDER $ICEBERG_FOLDER/iceberg.jpg.torrent
