# 📖 Hyperlit  
**Developer documentation that lives in your code**

Hyperlit is a developer tool that extracts specially marked documentation comments from source code and presents them in a searchable web interface.

It is designed to help teams keep architectural context, design decisions, and “why” explanations close to the code—and easy to find.

---

## Why another developer documentation tool?

Correct and up-to-date developer documentation (for example, explanations of *why* a particular approach was taken) is essential for sustainable and productive software development.

In practice, however, documentation often drifts out of sync with the code. External docs become outdated, incomplete, or forgotten entirely.

Hyperlit is built on a simple premise:

**Putting developer documentation directly in the source code makes it more likely to be found, reviewed, and kept up to date.**

---

## What are the benefits?

Keeping documentation aligned with code changes over the long term is one of the biggest challenges in software projects.

By placing documentation directly next to the relevant code (i.e. in the same file), Hyperlit provides several advantages:

1. **Better discoverability**  
   When reviewing or debugging code, explanations of *why* it is implemented a certain way are immediately visible—right where the code lives.

2. **More reliable updates**  
   When code changes, the corresponding documentation is hard to miss, making it more likely to be updated alongside the implementation.

3. **Automatic cleanup**  
   When functionality is removed, the related documentation disappears with it, preventing stale references to code that no longer exists.

---

## How does this work in practice?

Hyperlit works by extracting documentation blocks directly from your source code.

1. **Write documentation in the code**  
   Comment blocks that start with a special marker (for example `DOC` or `📖`) are treated as developer documentation.

   Example:

/// 📖 Why this cache exists
/// This cache avoids repeated database calls during startup,
/// significantly reducing application boot time.
function loadConfig() {
...
}


2. **Automatic extraction**  
Hyperlit scans the codebase and extracts all marked documentation blocks.

3. **Searchable web interface**  
All extracted documentation is displayed in a convenient web interface, where it can be browsed and searched.

4. **Source code linking**  
Each documentation block links back to its exact location in the source code, making it easy to jump between docs and implementation.

5. **Live updates**  
Documentation changes are hot-reloaded, so updates are reflected immediately as the code evolves.

---

## Who is Hyperlit for?

Hyperlit is particularly useful for:
- Teams maintaining long-lived codebases
- Projects with complex architectural or domain decisions
- Developers who want to document *why*, not just *what*

---

## Project status

Hyperlit is under active development. Features, APIs, and supported languages may evolve over time.