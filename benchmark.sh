#!/bin/bash

set -e

QUERYSETS=(
    ./lubm/queries-sparql.sparql
    ./test-queries.sparql
)

DATABASES=(
    ./data/University0_0-edited.nt
    # ./data/University0_all-edited.nt
    # ./data/University0-4_all-edited.nt
    # ./data/University0-9_all-edited.nt
)

SEMANTICS=(
    iterator
    collection
)

OPTIMIZER=(
    off
    arqpf
    arqpfj
    arqvc
    arqvcp
    random
    fixed
)

LOGFILE="evaluation/bench-$(date +%Y-%m-%d)-$1.log"
OUTFILE="evaluation/bench-$(date +%Y-%m-%d)-$1.csv"

echo "" > "$LOGFILE"
echo "" > "$OUTFILE"
cargo build --release

echo "number,query,semantics,optimizer,duration,measurer,rows,queryset,queries,database,triples" >> "$OUTFILE"

for i in $(seq 20); do
    for S in "${SEMANTICS[@]}"; do
        for O in "${OPTIMIZER[@]}"; do
            for DATABASE in "${DATABASES[@]}"; do
                for QUERYSET in "$QUERYSETS"; do

                    NUMBER_OF_QUERIES=$(( $(grep -c "^$" "$QUERYSET") + 1 ))

                    for q in $(seq $NUMBER_OF_QUERIES); do

                        NUMBER_OF_TRIPLES=$(wc -l "$DATABASE" | cut -d " " -f1)

                        if [[ $O == off ]]; then
                            if [[ $NUMBER_OF_TRIPLES > 15000 ]]; then
                                continue
                            fi
                        fi

                        START=$(date +%s.%N)
                        OUTPUT=$(RUST_LOG=info ./target/release/thesis parse $QUERYSET $DATABASE -s $S -o $O -n $q 2>&1; echo "EXIT CODE $?")
                        echo "$OUTPUT" >> $LOGFILE

                        ERROR=$(echo "$OUTPUT" | grep "EXIT CODE" | sed 's/EXIT CODE//g')
                        if [ $ERROR -ne 0 ]; then
                            echo "non-zero exit code $ERROR";
                            exit
                        fi

                        RUNTIME2=$(echo "$OUTPUT" | grep "Without optimizations " | sed 's/Without optimizations //g')
                        ROWS=$(echo "$OUTPUT" | grep "Total rows for query" | sed 's/Total rows for query//g' | cut -d " " -f2)

                        END=$(date +%s.%N)
                        RUNTIME1=$(echo "$END - $START" | bc -l)
                        printf "%3d,%3d,%s,%s,%5.8f,bash,%s,%s,%s,%s,%s\n" $i $q $S $O $RUNTIME1 $ROWS $QUERIES $NUMBER_OF_QUERIES $DATABASE $NUMBER_OF_TRIPLES >> "$OUTFILE"
                        printf "%3d,%3d,%s,%s,%s,self,%s,%s,%s,%s,%s\n" $i $q $S $O $RUNTIME2 $ROWS $QUERIES $NUMBER_OF_QUERIES $DATABASE $NUMBER_OF_TRIPLES >> "$OUTFILE"
                    done
                done
            done
        done
    done
done
