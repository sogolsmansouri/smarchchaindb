WORKDIR=$(pwd)

TM=$WORKDIR/docker/data/tendermint

docker-compose down

rm -rf $TM/data
mkdir $TM/data
mv $TM/priv_validator_state.json.backup $TM/data/priv_validator_state.json
sudo chmod -R 777 $TM/data

docker-compose up -d bigchaindb