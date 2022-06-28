#! /bin/bash

. .env

WORKDIR=$(mktemp -d -t hanoi.XXXXXXX)
function cleanup {
  if [ -d "$WORKDIR" ]; then
    rm -rf "$WORKDIR"
  fi
}
trap cleanup EXIT

ADDRESS=$1
echo Address: $ADDRESS
shift

METHOD=$1
echo Method: $METHOD
shift

PARAMS=$@
echo Params: $PARAMS

set -x
lotus chain invoke $ADDRESS $METHOD $PARAMS | tee $WORKDIR/output.log
{ set +x; } 2>/dev/null

OUTPUT=$(tail -1 $WORKDIR/output.log)

echo Decoded Output: $(echo $OUTPUT | base64 -d | sed 's,^.*State ,,')
