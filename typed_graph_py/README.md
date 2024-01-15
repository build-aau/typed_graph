# TypedGraph
Is a staticly typed graph library.
This is a cross compatible port of [TypedGraph][typed_graph-crates-io].

[![pypi](https://img.shields.io/pypi/v/typed_graph.svg)](https://pypi.org/pypi/typed_graph/)

TypedGraph is build to provide an easy to use interface for manipulating an maintiang graph with a rigid schema.

## Getting started
install using
```
pip install typed_graph
```

see the [example folder][examples-git] for information on how to create your first project using TypedGraph.

> Note that making and maintianing large static graphs can be time consuming, so using like [migration_handler][migration_handler-git] will make the library significantly easier to maintian 

## Development
To run a local instance of the library:
```
.../typed_graph_py> pip install -e .
```

This will import the library using a symlink, so changes in the directory will be propegated to the python installation

Now it can be used as normal
```
import type_graph
```

[typed_graph-crates-io]: https://crates.io/crates/typed_graph
[examples-git]: https://github.com/build-aau/typed_graph/tree/master/typed_graph_py/examples "example folder in git"
[migration_handler-git]: https://github.com/build-aau/migration_handler "migration_handler in git"