#!/bin/bash

OPENTRACKER_IP_ADDRESS="127.0.0.1"
OPENTRACKER_PORT=6969

opentracker i $OPENTRACKER_IP_ADDRESS p $OPENTRACKER_PORT

pkill opentracker
