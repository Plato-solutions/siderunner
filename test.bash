#!/bin/bash

integration_tests () {
   echo "Run integrational tests"
   cargo test --verbose --test '*' -- --test-threads=1
}

SERVER_PORT=8000
SELENIUM_PORT=4444

RUNNING_SELENIUM=$(netstat -tulpn | grep $SELENIUM_PORT)
RUNNING_SERVER=$(netstat -tulpn | grep $SERVER_PORT)

if [[ ! -z $RUNNING_SERVER ]] && [[ ! -z $RUNNING_SELENIUM ]]; then
   echo "Environment is already set up"
   integration_tests
else
   echo "Environment is not set"
   echo "Setting up environment"
   
   docker-compose up --no-start
   docker-compose start
   sleep 5
   
   integration_tests

   echo "Clean up environment"

   docker-compose stop
   docker-compose down
fi
