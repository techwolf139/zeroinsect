#!/bin/bash
cd /Users/mac/git/zeroinsect

echo "Starting broker..."
cargo run -- mqtt &
BROKER_PID=$!

sleep 2

echo "Starting client..."
cargo run --example mqtt_chat -- alice &
CLIENT_PID=$!

sleep 5

echo "Killing processes..."
kill $CLIENT_PID $BROKER_PID 2>/dev/null
wait

echo "Done"
