#!/bin/bash

# Start Docker daemon in background
dockerd &

# Wait for Docker daemon to be ready
sleep 5

# Execute the main application
exec /usr/local/bin/flytest
