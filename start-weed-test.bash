#! /bin/bash


mkdir -p /tmp/weed-volume
./test/weed master -mdir /tmp -port 8333 > /dev/null 2>&1 & 
MASTER=$!
./test/weed volume -mserver localhost:8333 -dir /tmp/weed-volume > /dev/null 2>&1 &
VOLUME=$!

read  -n 1 -p "Hit something to end execution"

echo Master PID
echo $MASTER
echo Volume PID
echo $VOLUME

kill $VOLUME
kill $MASTER