#! /bin/bash

set -e

ACTOR=$1
if [ -z "$ACTOR" ]; then
  echo "Usage: ./play.sh <actor-id>"
  exit 1
fi

WORKDIR=$(mktemp -d -t hanoi.XXXXXXX)
function cleanup {
  if [ -d "$WORKDIR" ]; then
    rm -rf "$WORKDIR"
  fi
}
trap cleanup EXIT

get_state() {
  ./invoke.sh $ACTOR 2 2>&1 | grep --line-buffered -v 'tee ' \
    | tee $WORKDIR/invoke.log
}

render_disc() {
  case $1 in
    1)
      echo '    \033[0;31m—\033[0m    '
      ;;
    2)
      echo '   \033[0;32m—--\033[0m   '
      ;;
    3)
      echo '  \033[0;34m--—--\033[0m  '
      ;;
    4)
      echo ' \033[0;35m----—--\033[0m '
      ;;
    5)
      echo '\033[0;36m------—--\033[0m'
      ;;
    .)
      echo '    |    '
      ;;
  esac
}

draw_towers() {
  OUTPUT=$1
  echo Towers:
  echo
  # { tower1: [5, 4], tower2: [3], tower3: [2, 1] }
  TOWER1=$(echo $OUTPUT | sed 's,^.*tower1: \[\([^]]*\)\].*$,\1,')
  IFS=', ' read -r -a TOWER1_ARRAY <<< "$TOWER1"
  TOWER2=$(echo $OUTPUT | sed 's,^.*tower2: \[\([^]]*\)\].*$,\1,')
  IFS=', ' read -r -a TOWER2_ARRAY <<< "$TOWER2"
  TOWER3=$(echo $OUTPUT | sed 's,^.*tower3: \[\([^]]*\)\].*$,\1,')
  IFS=', ' read -r -a TOWER3_ARRAY <<< "$TOWER3"

  for i in 4 3 2 1 0; do
    IFS='' TOWER1_DISC=$(render_disc ${TOWER1_ARRAY[i]:-.})
    IFS='' TOWER2_DISC=$(render_disc ${TOWER2_ARRAY[i]:-.})
    IFS='' TOWER3_DISC=$(render_disc ${TOWER3_ARRAY[i]:-.})
    echo -e "  " ${TOWER1_DISC}  ${TOWER2_DISC}  ${TOWER3_DISC}
  done
  echo "       1         2         3"
  echo
}

clear
echo "Reading initial state from Filecoin..."
echo
get_state

while true; do
  OUTPUT=$(sed -n 's/^Decoded Output: //p' $WORKDIR/invoke.log)
  if [ "$OUTPUT" = "{ tower1: [], tower2: [], tower3: [5, 4, 3, 2, 1] }" ]; then
    echo
    draw_towers "$OUTPUT"
    echo
    echo "!!!! You WIN !!!!"
    echo
    exit 0
  fi
  if [ -z "$OUTPUT" ]; then
    echo
    echo "Error? ... reloading state..."
    echo
    get_state
    continue
  fi
  clear
  echo Decoded Output: $OUTPUT
  echo

  draw_towers "$OUTPUT"

  read -p 'Move disc from: ' FROM
  read -p 'Move disc to: ' TO
  echo
  echo "Moving disc $FROM => $TO"
  echo
  ./invoke.sh $ACTOR 3 $(echo "$FROM$TO" | base64) 2>&1 | grep --line-buffered -v 'tee ' \
    | tee $WORKDIR/invoke.log
  sleep 2
done
