#!/bin/bash

set -e

QUERYSETS=(
    ./lubm/queries-sparql.sparql
    ./queries-for-dbpedia.sparql
    ./queries-for-lubm-with-filter.sparql
    # ./watdiv/bin/Release/queries-all-1-1.sparql
)

DATABASES=(
    ./lubm/University0_0-edited.nt
    # ./lubm/University0_all-edited.nt
    # ./lubm/University0-4_all-edited.nt
    # ./lubm/University0-9_all-edited.nt
    # ./dbpedia/sparql_2023-01-24-sm.nt
    ./dbpedia/sparql_2023-01-24-md.nt
    # ./dbpedia/sparql_2023-01-24-lg.nt
    ./watdiv/bin/Release/watdiv-1-unique.nt
    # ./watdiv/bin/Release/watdiv-10-unique.nt
)

SEMANTICS=(
    iterator
    # collection
)

LOGFILE="evaluation/explore-$(date +%Y-%m-%d)-$1.log"
OUTFILE="evaluation/explore-$(date +%Y-%m-%d)-$1.csv"

echo "" > "$LOGFILE"
echo "task,query,plan,rows,duration,queryset,queries,database,triples,optimizers,joins,scans,disjunct_joins,filters" > "$OUTFILE"

cargo build --release

for DATABASE in "${DATABASES[@]}"; do
    for QUERYSET in "${QUERYSETS[@]}"; do
        if [[ $DATABASE == *"lubm"* ]]; then
            if [[ $QUERYSET != *"lubm"* ]]; then
                continue
            fi
        fi

        if [[ $DATABASE == *"dbpedia"* ]]; then
            if [[ $QUERYSET != *"dbpedia"* ]]; then
                continue
            fi
        fi

        if [[ $DATABASE == *"watdiv-1"* ]]; then
            if [[ $QUERYSET != *"watdiv-1"* ]]; then
                continue
            fi
        fi

        if [[ $DATABASE == *"watdiv-5"* ]]; then
            if [[ $QUERYSET != *"watdiv-5"* ]]; then
                continue
            fi
        fi

        if [[ $DATABASE == *"watdiv-10"* ]]; then
            if [[ $QUERYSET != *"watdiv-1"* ]]; then
                continue
            fi
        fi

        NUMBER_OF_QUERIES=$(( $(grep -c "^$" "$QUERYSET") + 1 ))

        for q in $(seq $NUMBER_OF_QUERIES); do

            # if [[ $q -lt 7 ]]; then
            #     continue
            # fi

            echo $q $DATABASE $QUERYSET
            # continue

            OUTPUT=$(RUST_LOG=info RUST_BACKTRACE=1 ./target/release/thesis explore $QUERYSET $DATABASE -n $q 2>&1 | tee /dev/tty; echo "EXIT CODE $?")
            echo "$OUTPUT" >> "$LOGFILE"

            ERROR=$(echo "$OUTPUT" | grep "EXIT CODE" | sed 's/EXIT CODE//g')
            if [ $ERROR -ne 0 ]; then
                echo "non-zero exit code $ERROR";
                exit
            fi

            echo "$OUTPUT" | grep -E "^explore," | cat >> "$OUTFILE"
        done
    done
done
