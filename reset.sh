WORKDIR=$(pwd)

TM=$WORKDIR/docker/data/tendermint

docker-compose down

mv $TM/data/priv_validator_state.json $TM/priv_validator_state.json
rm -rf $TM/data
mkdir $TM/data
mv $TM/priv_validator_state.json $TM/data/priv_validator_state.json

docker-compose up -d bigchaindb