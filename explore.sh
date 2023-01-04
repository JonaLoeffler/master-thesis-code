#!/bin/bash

set -e

QUERYSETS=(
    ./lubm/queries-sparql.sparql
    # ./test-queries.sparql
)

DATABASES=(
    ./data/University0_0-edited.nt
    # ./data/University0_all-edited.nt
    # ./data/University0-4_all-edited.nt
    # ./data/University0-9_all-edited.nt
)

SEMANTICS=(
    iterator
    # collection
)

LOGFILE="evaluation/explore-$(date +%Y-%m-%d)-$1.log"
OUTFILE="evaluation/explore-$(date +%Y-%m-%d)-$1.csv"

echo "" > "$LOGFILE"
echo "task,query,plan,rows,duration,queryset,queries,database,triples,optimizers,joins,scans,disjunct_joins" > "$OUTFILE"

cargo build --release

for DATABASE in "${DATABASES[@]}"; do
    for QUERYSET in "$QUERYSETS"; do
        NUMBER_OF_QUERIES=$(( $(grep -c "^$" "$QUERYSET") + 1 ))

        for q in $(seq $NUMBER_OF_QUERIES); do

            # if [[ $q != $2 ]]; then
            #     continue
            # fi

            OUTPUT=$(RUST_LOG=info RUST_BACKTRACE=1 ./target/release/thesis explore $QUERYSET $DATABASE -n $q 2>&1 | tee /dev/tty; echo "EXIT CODE $?")
            echo "$OUTPUT" >> "$LOGFILE"

            ERROR=$(echo "$OUTPUT" | grep "EXIT CODE" | sed 's/EXIT CODE//g')
            if [ $ERROR -ne 0 ]; then
                echo "non-zero exit code $ERROR";
                exit
            fi

            echo "$OUTPUT" | grep -E "^explore," >> "$OUTFILE"
        done
    done
done
