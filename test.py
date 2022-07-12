#!/usr/bin/env python3
import math, sys

import platypus
from platypus import *

from collections import Counter, defaultdict

# path = "../../data/network-corpus/networks/{}.txt.gz"

# G = EditGraph.from_file(path.format('karate'))

print(K(5).remove_loops())
print(K(5).components())
print((K(5) + P(5)).components())

