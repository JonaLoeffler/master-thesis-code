#!/bin/bash

set -e

QUERYSETS=(
    ./lubm/queries-sparql.sparql
    ./queries-for-lubm-with-filter.sparql
    ./queries-for-dbpedia.sparql
    ./watdiv/bin/Release/queries-watdiv-10-all-1-1.sparql
    ./watdiv/bin/Release/queries-watdiv-20-all-1-1.sparql
)

DATABASES=(
    ./lubm/University0_0-edited.nt
    ./lubm/University0_all-edited.nt
    # ./lubm/University0-4_all-edited.nt
    # ./lubm/University0-9_all-edited.nt
    ./dbpedia/sparql_2023-01-24-sm.nt
    ./dbpedia/sparql_2023-01-24-md.nt
    ./dbpedia/sparql_2023-01-24-lg.nt
    ./watdiv/bin/Release/watdiv-10-unique.nt
    ./watdiv/bin/Release/watdiv-20-unique.nt
)

OPTIMIZER=(
    off
    arqpf
    arqpfc
    arqpfj
    arqpfjc
    arqvc
    arqvcp
    random
    fixed
)

FILTER_PUSHING=(
    ""
    "-c"
)

LOGFILE="evaluation/bench-$(date +%Y-%m-%d)-$1.log"
OUTFILE="evaluation/bench-$(date +%Y-%m-%d)-$1.csv"

echo "" > "$LOGFILE"
echo "" > "$OUTFILE"
cargo build --release

echo "number,query,semantics,optimizer,duration,measurer,rows,queryset,queries,database,triples,filter_pushing" >> "$OUTFILE"

S="iterator"
for i in $(seq 20); do
    for F in "${FILTER_PUSHING[@]}"; do
        for O in "${OPTIMIZER[@]}"; do
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

                    if [[ $DATABASE == *"watdiv"* ]]; then
                        if [[ $QUERYSET != *"watdiv"* ]]; then
                            continue
                        fi
                    fi

                    NUMBER_OF_QUERIES=$(( $(grep -c "^$" "$QUERYSET") + 1 ))

                    for q in $(seq $NUMBER_OF_QUERIES); do

                        NUMBER_OF_TRIPLES=$(wc -l "$DATABASE" | cut -d " " -f1)

                        TIMEOUT=20
                        START=$(date +%s.%N)
                        OUTPUT=$(RUST_LOG=info timeout --kill-after=0 $TIMEOUT ./target/release/thesis parse $QUERYSET $DATABASE -o $O -n $q $F 2>&1; echo "EXIT CODE $?")
                        echo "$OUTPUT" >> $LOGFILE

                        ERROR=$(echo "$OUTPUT" | grep "EXIT CODE" | sed 's/EXIT CODE//g')
                        if [[ $ERROR -ne 0 && $ERROR -ne 124 ]]; then
                            echo "non-zero exit code $ERROR";
                            exit
                        fi

                        RUNTIME2=$(echo "$OUTPUT" | grep "Without optimizations " | sed 's/Without optimizations //g')
                        ROWS=$(echo "$OUTPUT" | grep "Total rows for query" | sed 's/Total rows for query//g' | cut -d " " -f2)

                        if [[ $ERROR -eq 124 ]]; then
                            echo "non-zero exit code $ERROR, exited due to timeout";
                            RUNTIME2=$TIMEOUT
                            ROWS="N/A"
                        fi

                        END=$(date +%s.%N)
                        RUNTIME1=$(echo "$END - $START" | bc -l)
                        printf "%3d,%3d,%s,%s,%5.8f,bash,%s,%s,%s,%s,%s,%s\n" $i $q $S $O $RUNTIME1 $ROWS $QUERYSET $NUMBER_OF_QUERIES $DATABASE $NUMBER_OF_TRIPLES $F>> "$OUTFILE"
                        printf "%3d,%3d,%s,%s,%s,self,%s,%s,%s,%s,%s,%s\n" $i $q $S $O $RUNTIME2 $ROWS $QUERYSET $NUMBER_OF_QUERIES $DATABASE $NUMBER_OF_TRIPLES $F>> "$OUTFILE"
                    done
                done
            done
        done
    done
done
