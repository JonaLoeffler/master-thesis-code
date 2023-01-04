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
    with open("./lubm/queries-sparql.sparql") as file:
        lines = [line for line in file.readlines() if line[0] != "#"]
        queries = "".join(lines).split("\n\n")

    try:
        dataset = sys.argv[1]
    except:
        dataset = "lubm"

    start = time.time()
    for i, query in enumerate(queries):
        result = run(query, dataset).json()

        print(f"Query {i+1} results " + str(len(result["results"]["bindings"])))
    end = time.time()
    print("Finished in %.2fs" % ((end - start)))


if __name__ == "__main__":
    main()
