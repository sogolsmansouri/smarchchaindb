WORKDIR=$(pwd)

DBD=$WORKDIR/docker/data/_tmdbackup
TM=$WORKDIR/docker/data/tendermint

docker-compose down

rm -rf $TM/data
cp -r $DBD/data $TM/data
sudo chmod -R 777 $TM/data

docker-compose up -d bigchaindb