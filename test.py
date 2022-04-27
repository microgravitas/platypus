#!/usr/bin/env python3
import math, sys

import platypus
from platypus import *

from collections import Counter, defaultdict

path = "../../data/network-corpus/networks/{}.txt.gz"

G = EditGraph.from_file(path.format('ODLIS'))

G.remove_loops()
print(G)

DTF = DTFGraph.orient(G)

print(DTF)

D = DTF.domset(2)
print(D)
print(DTF)

covered = G.r_neighbourhood(D, 2)
print(len(covered))
print(len(G))



