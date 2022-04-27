#!/usr/bin/env python3
import functools, math, sys

import platypus
from platypus import *

from collections import Counter, defaultdict

# Simple decorator to 'register' algorithms
registry = {}
def algo(name):
    def algo_decorator(f):
        registry[name] = f;
        return f
    return algo_decorator

@algo("High-degree")
def degree(G):
    return G.degrees().rank(reverse=True)

@algo("Degeneracy")
def degeneracy(G):
    return G.degeneracy()[0]

@algo("Highdeg mod + degeneracy")
def highdeg_degeneracy(G):
    high_degs = G.degrees().rank(reverse=True)
    k = math.ceil(len(G) * .1)
    prefix = high_degs[:k]
    prefix += G[high_degs[k:]].degeneracy()[0]
    return prefix

@algo("Core num + high-degree")
def highdeg_degeneracy(G):
    order, corenums = G.degeneracy()
    degs = G.degrees()
    n = len(G)
    # We want high corenums/high degree first. We also add
    # a tie-break using the degeneracy order.
    vdata = [(corenums[v], degs[v], n-i, v) for i,v in enumerate(order)]
    vdata.sort(reverse=True)
    return [e[-1] for e in vdata]


path = "../../data/network-corpus/networks/{}.txt.gz"


# G = EditGraph.from_file(path.format('bergen'))
# G = EditGraph.from_file(path.format('ODLIS'))
G = EditGraph.from_file(path.format('digg'))

G.remove_loops()
print(G)

for name, f in registry.items():
    print(f"Algorithm '{name}'")
    order = f(G)

    OG = G.to_ordered(order)
    for r in range(1,4):
        wcol = OG.wreach_sizes(r)
        print(f"  Wcol{r} = {wcol.max()} (avg. {wcol.mean():.1f})")




