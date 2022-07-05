#!/usr/bin/env python3
import math, sys

import platypus
from platypus import *

from collections import Counter, defaultdict

path = "../../data/network-corpus/networks/{}.txt.gz"

G = K(1,10)

print(G, G.is_bipartite())


G = EditGraph()
G.add_edge(0,1)
G.add_edge(1,2)
G.add_edge(2,0)
print(G, G.is_bipartite())

G = EditGraph.from_file(path.format('karate'))
print(G.edges())
print(G, G.is_bipartite())

# path = "../../data/network-corpus/networks/{}.txt.gz"

# G = EditGraph.from_file(path.format('bergen'))

# G.remove_loops()  
# print(G)


# DTF = DTFGraph.orient(G)

# print(DTF)

# r = 2
# D = DTF.domset(r)
# print(f"{r}-domset: {D}")

# covered = G.r_neighbourhood(D, 2)
# print(len(G) == len(covered))

# dist = DTF.small_distance(1, 52)
# print(dist)


