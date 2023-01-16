#!/bin/bash

TORRENT_FILE=$1         # given torrent file  as the first argument
WORKING_DIR=$2          # given directory where the is going to be saved
WORKING_DIR_ARIAS=$3    # given directory where the file to upload is stored

gnome-terminal -- aria2c -V -d $WORKING_DIR_ARIAS/aria1-4/ $TORRENT_FILE --listen-port=2002 
gnome-terminal -- aria2c -V -d $WORKING_DIR_ARIAS/aria5-8/ $TORRENT_FILE --listen-port=2001 
gnome-terminal -- aria2c -V -d $WORKING_DIR_ARIAS/aria9-11/ $TORRENT_FILE --listen-port=2003 



gnome-terminal -- cargo run -- $TORRENT_FILE $WORKING_DIR --debug --mock
