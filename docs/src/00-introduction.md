# Introduction

**hyperlit is a system for embedding documentation in code, thus making it easy to write and maintain documentation
for a software project.**

Rather than writing documentation in a separate file, you can embed it directly into your code:

Example:

```rust
/*📖
title: Adding new widgets

Here's how to add a new widgets....
*/
```

## Advantages

This has the following advantages:

- It's easy to add documentation to existing code
- A higher chance of keeping documentation up to date, since it's part of the code
- It removes the cognitive load of deciding where to put documentation
- Removing source files automatically removes the corresponding documentation, thereby reducing confusion caused by stale documentation

## Goals

The primary goals of hyperlit are:

1. Make it easy to add documentation
2. Support for multiple languages
3. Support for multiple documentation formats

## Values

The guiding values of hyperlit are:

1. User-friendliness: Tools should be a joy to use
2. Productivity: Documentation should make us more productive, not be a chore
3. Simplicity: Prefer simple tools over complex ones

## Observations and conclusions from my personal documentation experience

1. Documentation *requires* a great search mechanism to be useful

   If your documentation search is great, it does not matter where you put it: flat search beats deeply nested
   hierarchies
2. The easier it is to write documentation, the more likely it is going to be done
3. The more obvious existing documentation is, the more likely it is going to be kept in sync with the code
4. Rather than putting documentation in its own cupboard, let's put it *in* the code

§{@include_rest}