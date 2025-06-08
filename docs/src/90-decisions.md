# Decision records

This section contains decision records for the project.

The purpose of these records is to document decisions that have been made about the project.

## DR-0000 Use decision records to document decisions

Status: Approved \
Date: 2025-06-08

### Decision

To document the values, structure, architecture and of the project, we will use decision records to document
**why** things are the way they are, and what alternatives have been considered.

### Context

Source code mostly conveys the **How** of a solution, but is often insufficient to explain the **Why** behind it,
and usually never the *alternatives* that were considered, but ultimately rejected.

A lack of understanding of these "Why's" can lead to incorrect assumptions, duplicate effort, inconsistent
structures and behaviors, as well as an overall lack of vision.

### Consequences

For all major decisions, we will use decision records to document them.

These decision records will be checked in to the source control repository.

The decision records should be written in a way that is clear, concise and easy to understand.

The format should be:

> Title: DR-{sequential number} {title}\
> Status: {Decisions status, one of: In Progress, Approved, Rejected, Superseded}\
> Date: {Date of last update in YYYY-MM-DD format}
>
> ### Decision
>
> {What is the decision?}
>
> ### Context
>
> {What question does this decision answer?}
>
> ### Consequences
>
> {What are the consequences of this decision?}
>
> ### Considered Alternatives
>
> {What alternatives were considered? Why were they rejected?}
>