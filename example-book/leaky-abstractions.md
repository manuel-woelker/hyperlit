# The Law of Leaky Abstractions

**Statement:** "All non-trivial abstractions, to some degree, are leaky."

**Proposed by:** Joel Spolsky (2002)

**Key Insight:** Abstractions in software development often 'leak' details they're supposed to hide, forcing developers to understand the underlying implementation.

**Examples:**
- SQL abstracts database operations but requires understanding of query optimization
- ORMs abstract database access but may require knowledge of SQL for complex queries
- Virtual memory abstracts physical memory but can lead to performance issues

**Implications:**
- No abstraction can completely hide complexity
- Developers often need to understand lower-level details
- Over-reliance on abstractions can lead to unexpected behaviors
- Important to understand at least one level below your current abstraction
