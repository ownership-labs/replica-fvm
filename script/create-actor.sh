#! /bin/bash

. .env

WORKDIR=$(mktemp -d -t hanoi.XXXXXXX)
function cleanup {
  if [ -d "$WORKDIR" ]; then
    rm -rf "$WORKDIR"
  fi
}
trap cleanup EXIT

CID=$1
echo Code CID: $1
shift
echo Encoded Params: "$@"

set -x
lotus chain create-actor $CID $@ | tee $WORKDIR/output.log
{ set +x; } 2>/dev/null

ID=$(cat $WORKDIR/output.log | sed -n 's,^ID Address: ,,p')

echo Next step:
echo ./invoke.sh $ID '<method num>' '<encoded-params>'
echo
echo
echo "eg. ./invoke.sh $ID 2   # Get state"
echo "    ./invoke.sh $ID 3" '$(echo 12 | base64)   # Move disc from tower 1 to tower 2'
echo "    ./play.sh $ID   # GUI for game written in bash"
