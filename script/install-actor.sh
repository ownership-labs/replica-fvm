#! /bin/bash

. .env

WORKDIR=$(mktemp -d -t hanoi.XXXXXXX)
function cleanup {
  if [ -d "$WORKDIR" ]; then
    rm -rf "$WORKDIR"
  fi
}
trap cleanup EXIT

set -x
lotus chain install-actor target/debug/wbuild/hanoi_actor_1/hanoi_actor_1.compact.wasm | tee $WORKDIR/output.log
{ set +x; } 2>/dev/null

CID=$(cat $WORKDIR/output.log | sed -n 's,^Actor Code CID: ,,p')

echo Next step:
#echo ./create-actor.sh $CID '<encoded-params>'
echo ./create-actor.sh $CID
