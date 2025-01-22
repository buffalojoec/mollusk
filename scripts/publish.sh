#!/bin/bash

set -e
set -u
set -o pipefail

CRATES=(
  "mollusk-svm-error"
  "mollusk-svm-keys"
  "mollusk-svm-fuzz-fixture"
  "mollusk-svm-fuzz-fixture-firedancer"
  "mollusk-svm"
  "mollusk-svm-bencher"
  "mollusk-svm-programs-memo"
  "mollusk-svm-programs-token"
)

publish_crate() {
  local crate=$1
  local args=$2

  echo "Publishing $crate..."

  cargo publish -p $crate --token $TOKEN $args

  echo "$crate published successfully!"

  sleep 5
}

# Publish each crate in order
for crate in "${CRATES[@]}"; do
  publish_crate "$crate" $1
done

echo "All crates published successfully!"

