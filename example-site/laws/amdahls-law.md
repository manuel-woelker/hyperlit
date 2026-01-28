---
date: 2025-11-14
author: "Gene Amdahl"
---

# Amdahl's Law #law

**Statement:** The theoretical speedup of a task using multiple processors is limited by the portion of the task that cannot be parallelized.

**Proposed by:** Gene Amdahl (1967)

**Formula:** Speedup = 1 / [(1 - P) + (P/N)]

- P = proportion of the task that can be parallelized
- N = number of processors

**Key Insight:** There's a limit to how much you can speed up a task by adding more processors, due to the sequential portion of the program.

**Implications:**

- Diminishing returns when adding more processors
- Focus should be on optimizing the sequential portion
- Important for understanding parallel computing limitations

**Table**

| Alpha bravo charlie |              delta |
|---------------------|-------------------:|
| Echo                | Foxtrot golf hotel |