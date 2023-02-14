#!/bin/sh
cargo build -r
if [ $? -eq 0 ]; then
   echo "Build Worked"
else
   echo "Build Failed"
   exit
fi

sudo docker-compose down
rm -f ./disk.img
sudo dd if=/dev/zero of=disk.img bs=1M count=1200
sudo docker-compose up -d --build
sudo docker container logs app
sudo docker container exec -it app /bin/bash
# sudo docker-compose down