#!/bin/env python
import sys
import time
import requests
import urllib.parse


def run(query: str, dataset: str):
    return requests.post(
        f"http://localhost:3030/{dataset}/sparql?query=" + urllib.parse.quote(query)
    )


def main():
    try:
        queryset = sys.argv[1]
    except:
        queryset = "./lubm/queries-sparql.sparql"

    try:
        dataset = sys.argv[2]
    except:
        dataset = "lubm"

    with open(queryset) as file:
        lines = [line for line in file.readlines() if line[0] != "#"]
        queries = "".join(lines).split("\n\n")

    start = time.time()
    for i, query in enumerate(queries):
        result = run(query, dataset).json()

        print(f"Query {i+1} results " + str(len(result["results"]["bindings"])))
    end = time.time()
    print("Finished in %.2fs" % ((end - start)))


if __name__ == "__main__":
    main()
