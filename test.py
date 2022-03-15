#!/usr/bin/env python3

import platypus
from platypus import *

from collections import Counter

path = "../../data/network-corpus/networks"



# G = EditGraph.from_file(f"{path}/digg.txt.gz")
G = EditGraph.from_file(f"{path}/bergen.txt.gz")

OG = OrdGraph.by_degeneracy(G)
print(OG)

order = G.degrees().rank()
OG = G.to_ordered(order)
print(OG.wreach_sizes(3).values())




