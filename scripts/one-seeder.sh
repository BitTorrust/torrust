#!/bin/bash

OPENTRACKER_IP_ADDRESS="127.0.0.1"
OPENTRACKER_PORT=6969

TORRENT_FILE=$1                     # given torrent file  as the first argument
WORKING_DIR=$2                      # given directory where the file to upload is stored
PEER_BITTORRENT_DOWNLOAD_PORT=$3    # given port as the third argument

opentracker -i $OPENTRACKER_IP_ADDRESS -p $OPENTRACKER_PORT & \
aria2c -V -d $WORKING_DIR $TORRENT_FILE --listen-port=$PEER_BITTORRENT_DOWNLOAD_PORT

pkill opentracker