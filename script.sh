#!/usr/bin/env bash

first=c4096edb47f3a07e4f9d670c7edff564329b82f9
last=01d14f0a82c860849e9cfb5884f5b54e8486b248
subdir=subdir

git filter-branch --tree-filter '
  first='"$first"'
  last='"$last"'

  subdir='"$subdir"'
  log_file=/tmp/filter.log

  [ "$GIT_COMMIT" = "$first" ] && seen_first=true

  if [ "$seen_first" = "true" ] && [ "$seen_last" != "true" ]; then
    echo "=== $GIT_COMMIT: making changes"
    files=$(git ls-tree --name-only $GIT_COMMIT)
    mkdir -p $subdir
    for i in $files; do
      mv $i $subdir || echo "ERR: mv $i $subdir failed"
    done
  else
    echo "=== $GIT_COMMIT: ignoring"
  fi \
    >> $log_file

  [ "$GIT_COMMIT" = "$last" ] && seen_last=true

  status=0  # tell tree-filter never to fail
'
