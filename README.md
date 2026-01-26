# 📖 Hyperlit

Hyperlit is a tool for extracting, displaying and searching integrated developer documentation.

## Why another developer documentation tool?

Hyperlit builds on the following premise:

**Putting this developer documentation right in the source code makes it more likely to be found and kept up-to-date.**

Correct and up-to-date developer documentation (i.e. explanations *why* an approach was taken) is essential for sustainable and productive software 
development.

## What are the benefits?

Experience has shown that keeping the documentation up-to-date with code-changes over the long term is one of the key challenges when working with software 
projects.

Putting the documentation as close to the code as possible (i.e. in the same file) has the following benefits:

1. When reviewing or debugging the code, explanations on why it is implemented this way are easy to find, because it is *right there.*
2. When changing the code, it is more likely that the corresponding documentation will be changed as well, because you basically trip over it.
3. When removing functionality, the corresponding documentation is automatically removed as well, since it lives in the same file. This prevents outdated documentation references to functionality that no longer exists.

## How does this work in practice?

1. Comment blocks that start with special marker (e.g. "DOC" or "📖") are extracted by hyperlit.
2. All these documentation blocks can be viewed and searched via a convenient web interface.
3. These blocks contain links to the source code, so users can quickly navigate to the corresponding source location.
4. Documentation-updates are hot-reloaded on any changes.

