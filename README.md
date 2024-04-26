# raphle
Blazingly fast in-memory graph database available over HTTP

## Motivation
Graph databases are slow and clunky. Given their large operational overhead, they don't make much sense for smaller projects and services that need to run graph computations. Furthermore, the structure of a graph is often more important than the data stored in it. `raphle` takes the approach of "form over substance", opting to serve graph analytics and requests over HTTP once the graph is loaded into memory.

The goal of `raphle` is to be reliable, fast, and easy to use. `raphle` aims to achieve this by: 
1. Relying on Rust's memory management model and fearless concurrency principles.
2. Utilizing a low-level asynchronous runtime that minimizes the memory footprint of even really large graphs.
3. Loading any graph type from any valid adjacency list. Simply define a data source that follows a `(source: u32, target: u32, edge_count: u32)` standard.

## `raphle` is currently under development
`raphle` is currently pre-alpha, and as such is expected to undergo drastic and breaking changes to it's API. If you'd like to help get `raphle` to its first runnable release, check out the Issues tab.
